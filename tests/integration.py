"""Real coordinator + two local agent processes + real Docker integration checks.

Uses temporary credentials and only cleans up resources labeled with its node IDs.
Set CLOUDLAB_TEST_APPS=1 to additionally test Jupyter and code-server HTTP/WS relays.
"""
import base64
import http.client
import json
import os
from pathlib import Path
import socket
import struct
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
BIN = ROOT / "target/debug/cloudlab"

def port():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]

def wait_for(fn, seconds=90):
    deadline = time.monotonic() + seconds
    last = None
    while time.monotonic() < deadline:
        try:
            value = fn()
            if value:
                return value
        except (OSError, urllib.error.URLError) as e:
            last = e
        time.sleep(1)
    raise AssertionError(f"Timed out waiting for operation: {last}")

class API:
    def __init__(self, origin):
        self.origin, self.cookie = origin, ""

    def call(self, path, body=None, method=None, expected=200, csrf=True):
        headers = {"Content-Type": "application/json", "Cookie": self.cookie}
        if csrf:
            headers["X-CloudLab-Client"] = "web"
        req = urllib.request.Request(self.origin + "/api" + path,
              data=None if body is None else json.dumps(body).encode(),
              headers=headers, method=method or ("GET" if body is None else "POST"))
        try:
            response = urllib.request.urlopen(req, timeout=15)
        except urllib.error.HTTPError as e:
            response = e
        value = json.loads(response.read())
        assert response.status == expected, (path, response.status, expected, value)
        if response.headers.get("Set-Cookie"):
            self.cookie = response.headers["Set-Cookie"].split(";")[0]
        return value

def docker(*args):
    return subprocess.check_output(["docker", *args], text=True, stderr=subprocess.DEVNULL).strip()

def gateway(url, cookie="", path="/", origin=None):
    parsed = urllib.parse.urlsplit(url)
    c = http.client.HTTPConnection("127.0.0.1", parsed.port, timeout=65)
    headers = {"Host": parsed.netloc, "Cookie": cookie}
    if origin:
        headers["Origin"] = origin
    c.request("GET", path, headers=headers)
    response = c.getresponse()
    status, headers, body = response.status, dict(response.getheaders()), response.read()
    c.close()
    return status, headers, body

def ws_handshake(url, path, cookie, origin):
    parsed = urllib.parse.urlsplit(url)
    sock = socket.create_connection(("127.0.0.1", parsed.port), timeout=15)
    key = base64.b64encode(os.urandom(16)).decode()
    sock.sendall((f"GET {path} HTTP/1.1\r\nHost: {parsed.netloc}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\nOrigin: {origin}\r\nCookie: {cookie}\r\n\r\n").encode())
    data = b""
    while b"\r\n\r\n" not in data:
        data += sock.recv(4096)
    sock.close()
    return int(data.split(b" ")[1])

def main():
    assert BIN.exists(), "Run cargo build -p cloudlab first"
    coord_port, app_port = port(), port()
    origin = f"http://127.0.0.1:{coord_port}"
    api = API(origin)
    processes, node_ids = [], []
    with tempfile.TemporaryDirectory(prefix="cloudlab-test-") as temp:
        directory = Path(temp)
        log = open(directory / "process.log", "w+")
        try:
            processes.append(subprocess.Popen([str(BIN), "serve", "--bind", f"127.0.0.1:{coord_port}", "--app-bind", f"127.0.0.1:{app_port}", "--app-url", f"http://{{workspace}}.localhost:{app_port}", "--data-dir", str(directory / "hub"), "--web-dir", str(ROOT / "dist")], stdout=log, stderr=log))
            wait_for(lambda: api.call("/health"))
            api.call("/state", expected=401)
            api.call("/login", {"token": "invalid"}, expected=401)
            api.call("/login", {"token": "invalid"}, expected=403, csrf=False)
            api.call("/login", {"token": (directory / "hub/admin-token").read_text()})
            lab = api.call("/state")["labs"][0]["id"]
            other = api.call("/labs", {"name": "Other lab", "description": "Scope boundary"})["id"]
            print("PASS authentication, CSRF rejection, and lab creation", flush=True)
            for index, lab_id in enumerate([lab, other]):
                enrollment = api.call("/enrollments", {"lab_id": lab_id, "name": f"Test node {index}"})["token"]
                env = dict(os.environ, CLOUDLAB_ENROLLMENT=enrollment)
                processes.append(subprocess.Popen([str(BIN), "agent", "--coordinator", origin, "--data-dir", str(directory / f"node-{index}")], env=env, stdout=log, stderr=log))
                node = wait_for(lambda: next((n for n in api.call("/state")["nodes"] if n["lab_id"] == lab_id and n["docker"]), None))
                node_ids.append(node["id"])
                api.call("/agent/enroll", {"token": enrollment, "platform": "Linux", "arch": "x86_64", "cpus": 1, "memory_mb": 512}, expected=401)
            telemetry = wait_for(lambda: next((n for n in api.call("/state")["nodes"] if n["id"] == node_ids[0] and n.get("metrics") and n.get("history")), None))
            assert 0 <= telemetry["metrics"]["cpu_usage"] <= 100
            assert telemetry["metrics"]["memory_total_mb"] == telemetry["memory_mb"]
            assert "credential_hash" not in telemetry
            print("PASS two real enrolled agents, pairing, node telemetry and history", flush=True)

            viewer_token = api.call("/access", {"lab_id": lab, "name": "Viewer", "role": "viewer"})["token"]
            viewer = API(origin)
            viewer.call("/login", {"token": viewer_token})
            state = viewer.call("/state")
            assert len(state["labs"]) == len(state["nodes"]) == 1
            viewer.call("/labs", {"name": "Forbidden lab"}, expected=403)
            config = {"lab_id": lab, "node_id": node_ids[0], "name": "Integration console", "template": "terminal", "cpus": 1, "memory_mb": 512, "network": False}
            viewer.call("/workspaces", config, expected=403)
            api.call("/workspaces", dict(config, template="arbitrary/image"), expected=400)
            api.call("/workspaces", dict(config, cpus=9999), expected=400)
            api.call("/workspaces", dict(config, gpu_ids=["/dev/mem"]), expected=400)
            api.call("/workspaces", dict(config, network=True), expected=400)
            operator_token = api.call("/access", {"lab_id": lab, "name": "Operator", "role": "operator"})["token"]
            operator = API(origin)
            operator.call("/login", {"token": operator_token})
            operator.call("/workspaces", dict(config, lab_id=other, node_id=node_ids[1]), expected=403)
            print("PASS viewer/operator permissions, cross-lab denial, template and budget validation", flush=True)

            def workspace_status(wid, status):
                w = next((w for w in api.call("/state")["workspaces"] if w["id"] == wid), None)
                if w and w["status"] == "error":
                    raise AssertionError(w["error"])
                return w if w and w["status"] == status else None

            def action(wid, action, command=""):
                result = operator.call(f"/workspaces/{wid}/actions", {"action": action, "command": command})
                return wait_for(lambda: (j if j["status"] in ["done", "failed"] else None) if (j := operator.call(f"/jobs/{result['id']}")) else None)

            wid = operator.call("/workspaces", config)["id"]
            wait_for(lambda: workspace_status(wid, "running"))
            measured = wait_for(lambda: next((w for w in api.call("/state")["workspaces"] if w["id"] == wid and w.get("metrics") and w.get("history")), None))
            assert 0 <= measured["metrics"]["cpu_usage"] <= 100
            assert measured["metrics"]["memory_total_mb"] == 512
            assert measured["metrics"]["pids"] > 0
            assert measured["metrics"]["gpus"] == []
            print("PASS real Docker CPU, memory, process and I/O telemetry", flush=True)
            info = json.loads(docker("inspect", f"cloudlab-{wid}"))[0]
            host = info["HostConfig"]
            assert host["ReadonlyRootfs"] and host["Privileged"] is False
            assert host["CapDrop"] == ["ALL"] and host["PidsLimit"] == 256
            assert host["Memory"] == host["MemorySwap"] == 512 * 1024 * 1024
            assert host["NanoCpus"] == 1_000_000_000
            assert info["Config"]["User"] == "1000:1000"
            assert all(m["Type"] != "bind" for m in info["Mounts"])
            assert host["NetworkMode"] == "none"
            assert not host["PortBindings"]
            result = action(wid, "exec", "id -u; test ! -e /var/run/docker.sock && echo NO_DOCKER_SOCKET; test ! -w /etc && echo READ_ONLY_SYSTEM; printf 'persistent-data' > /home/lab/probe")
            assert result["status"] == "done" and "1000" in result["output"] and "NO_DOCKER_SOCKET" in result["output"] and "READ_ONLY_SYSTEM" in result["output"], result
            assert action(wid, "stop")["status"] == "done"
            wait_for(lambda: workspace_status(wid, "stopped"))
            assert action(wid, "start")["status"] == "done"
            result = action(wid, "exec", "cat /home/lab/probe")
            assert result["output"] == "persistent-data", result
            print("PASS real container creation, isolation flags, non-root commands, stop/resume, persistent storage", flush=True)

            if os.environ.get("CLOUDLAB_TEST_APPS") == "1":
                for template in ["jupyter", "code"]:
                    app = operator.call("/workspaces", dict(config, name=f"Test {template}", template=template, memory_mb=1024))["id"]
                    wait_for(lambda: workspace_status(app, "running"))
                    launch = operator.call(f"/workspaces/{app}/open", {})["url"]
                    parsed = urllib.parse.urlsplit(launch)
                    status, headers, _ = gateway(launch, path=f"/?{parsed.query}")
                    assert status == 303, status
                    assert headers["location"] == "/_cloudlab/"
                    assert "SameSite=Lax" in headers["set-cookie"]
                    assert "HttpOnly" in headers["set-cookie"]
                    cookie = headers["set-cookie"].split(";")[0]
                    assert gateway(launch, path=f"/?{parsed.query}")[0] == 401
                    assert gateway(launch)[0] == 401
                    shell = gateway(launch, cookie, "/_cloudlab/")
                    assert shell[0] == 200 and b"CloudLab" in shell[2]
                    assert shell[1]["x-frame-options"] == "SAMEORIGIN"
                    info = json.loads(gateway(launch, cookie, "/_cloudlab/workspace.json")[2])
                    assert info["template"] == template and info["cpus"] == 1
                    assert info["node"] == "Test node 0"
                    assert info["status"] == "running" and info["node_online"]
                    assert b"metric-charts" in shell[2]
                    assert "metrics" in info and "history" in info
                    assert info["gpus"] == [] and "credential_hash" not in info
                    measured_info = wait_for(lambda: (v if v.get("metrics") else None) if (v := json.loads(gateway(launch, cookie, "/_cloudlab/workspace.json")[2])) else None)
                    assert measured_info["metrics"]["memory_total_mb"] == 1024
                    assert gateway(launch, path="/_cloudlab/workspace.json")[0] == 401
                    app_origin = f"{parsed.scheme}://{parsed.netloc}"
                    if template == "jupyter":
                        def app_ready():
                            response = gateway(launch, cookie, "/api/status")
                            if response[0] != 200:
                                print("App waiting:", response[0], response[2][:300].decode(errors="replace"), flush=True)
                            return response if response[0] == 200 else None
                        status, _, body = wait_for(app_ready, seconds=30)
                        assert "kernels" in json.loads(body)
                        assert ws_handshake(launch, "/api/events/subscribe", cookie, "http://evil.example") == 403
                        assert ws_handshake(launch, "/api/events/subscribe", cookie, app_origin) == 101
                    else:
                        result = wait_for(lambda: (r if r[0] in [200, 302] else None) if (r := gateway(launch, cookie)) else None)
                        assert result[0] in [200, 302]
                        product = json.loads(docker("run", "--rm", "--read-only", "--network", "none", "--entrypoint", "cat", "cloudlab/code:2", "/usr/lib/code-server/lib/vscode/product.json"))
                        asset = f"/stable-{product['commit']}/static/out/vs/code/browser/workbench/workbench.js"
                        status, _, javascript = gateway(launch, cookie, asset)
                        assert status == 200 and len(javascript) > 8 * 1024 * 1024, (status, len(javascript), javascript[:100])
                        print("PASS code-server large workbench bundle (over 8 MiB)", flush=True)
                    print(f"PASS {template} gateway, one-time ticket, isolated origin and authentication", flush=True)
                    assert action(app, "delete")["status"] == "done"

            # Local agent stop keeps the running container and the saved pairing.
            subprocess.run([str(BIN), "agent", "stop", "--data-dir", str(directory / "node-0")], check=True)
            processes[1].wait(timeout=10)
            assert json.loads(docker("inspect", f"cloudlab-{wid}"))[0]["State"]["Running"]
            processes[1] = subprocess.Popen([str(BIN), "agent", "start", "--data-dir", str(directory / "node-0")], stdout=log, stderr=log)
            wait_for(lambda: (directory / "node-0/agent-run").exists())
            assert action(wid, "delete")["status"] == "done"
            assert docker("volume", "inspect", f"cloudlab-{wid}-home")
            # Delete enrollment revokes only this node and permits a fresh pairing
            # in the same directory. No workspace volume is removed by the CLI.
            subprocess.run([str(BIN), "agent", "delete", "--data-dir", str(directory / "node-0")], check=True)
            processes[1].wait(timeout=10)
            assert not (directory / "node-0/credentials.json").exists()
            assert next(n for n in api.call("/state")["nodes"] if n["id"] == node_ids[0])["revoked"]
            assert not next(n for n in api.call("/state")["nodes"] if n["id"] == node_ids[1])["revoked"]
            assert docker("volume", "inspect", f"cloudlab-{wid}-home")
            enrollment = api.call("/enrollments", {"lab_id": lab, "name": "Repaired node"})["token"]
            processes[1] = subprocess.Popen([str(BIN), "agent", "--coordinator", origin, "--data-dir", str(directory / "node-0")], env=dict(os.environ, CLOUDLAB_ENROLLMENT=enrollment), stdout=log, stderr=log)
            repaired = wait_for(lambda: next((n for n in api.call("/state")["nodes"] if n["name"] == "Repaired node" and n["docker"]), None))
            assert repaired["id"] not in node_ids
            node_ids.append(repaired["id"])
            print("PASS agent stop, saved-pairing resume, enrollment deletion, scoped revocation, same-folder re-pairing, retained volume", flush=True)
            access_id = next(a["id"] for a in api.call("/state")["access"] if a["name"] == "Viewer")
            api.call(f"/access/{access_id}", {}, "DELETE")
            viewer.call("/state", expected=401)
            api.call(f"/nodes/{node_ids[1]}", {}, "DELETE")
            print("PASS removal retains data, access revocation invalidates sessions, node revocation", flush=True)
            # Persistence must survive a coordinator restart.
            processes[0].terminate()
            processes[0].wait(timeout=5)
            processes[0] = subprocess.Popen([str(BIN), "serve", "--bind", f"127.0.0.1:{coord_port}", "--app-bind", f"127.0.0.1:{app_port}", "--app-url", f"http://{{workspace}}.localhost:{app_port}", "--data-dir", str(directory / "hub"), "--web-dir", str(ROOT / "dist")], stdout=log, stderr=log)
            assert wait_for(lambda: len(api.call("/state")["labs"]) == 2)
            print("PASS durable state and session recovery after restart", flush=True)
        except Exception:
            log.flush()
            for node in node_ids:
                for cid in docker("ps", "-aq", "--filter", f"label=cloudlab.node={node}").splitlines():
                    subprocess.run(["docker", "logs", "--tail", "15", cid])
            print((directory / "process.log").read_text()[-7000:])
            raise
        finally:
            for process in processes:
                process.terminate()
            for process in processes:
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
            # Only this test's specifically labeled resources are eligible.
            for node in node_ids:
                for cid in docker("ps", "-aq", "--filter", f"label=cloudlab.node={node}").splitlines():
                    volumes = json.loads(docker("inspect", cid))[0]["Mounts"]
                    docker("rm", "-f", cid)
                    for volume in volumes:
                        if volume["Type"] == "volume" and volume["Name"].startswith("cloudlab-"):
                            docker("volume", "rm", volume["Name"])
                for net in docker("network", "ls", "-q", "--filter", f"label=cloudlab.node={node}").splitlines():
                    docker("network", "rm", net)
            # Volumes for workspaces deleted through the API are retained by design.
            # Remove only those created by this run, using the persisted workspace jobs.
            state_path = directory / "hub/state.json"
            if state_path.exists():
                ids = {j["workspace"]["id"] for j in json.loads(state_path.read_text())["jobs"] if j["node_id"] in node_ids}
                for wid in ids:
                    subprocess.run(["docker", "volume", "rm", f"cloudlab-{wid}-home"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            log.close()

if __name__ == "__main__":
    main()
