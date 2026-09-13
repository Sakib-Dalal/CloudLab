#!/usr/bin/env python3
"""Exercise generated routing using Docker and an isolated local test CA.

Build first: docker build -t cloudlab-cloud-check:local .
No public certificates, cloud metadata, DNS queries, or host services are used.
"""
import http.client
import importlib.util
import json
import socket
import ssl
import subprocess
import tempfile
import time
import uuid
from pathlib import Path
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("cloud", ROOT / "scripts/cloud-access.py")
cloud = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cloud)


def main():
    project = "cloudlab-gateway-test-" + uuid.uuid4().hex[:10]
    with tempfile.TemporaryDirectory(prefix=project) as directory:
        output = Path(directory)
        plan = cloud.addresses("8.8.8.8")  # Name fixture; all sockets connect to loopback.
        host = urlsplit(plan["dashboard"]).hostname
        config = cloud.compose_config(plan, ROOT, output)
        config["name"] = project
        coordinator = config["services"]["coordinator"]
        del coordinator["build"]
        coordinator["image"] = "cloudlab-cloud-check:local"
        coordinator["ports"] = ["127.0.0.1::8088"]
        config["services"]["gateway"]["ports"] = ["127.0.0.1::80", "127.0.0.1::443"]
        (output / "compose.yaml").write_text(json.dumps(config))
        (output / "Caddyfile").write_text(cloud.caddyfile(plan).replace("{\n", "{\n    local_certs\n", 1))

        def compose(*args):
            return subprocess.check_output(["docker", "compose", "-p", project, "-f", str(output / "compose.yaml"), *args],
                                           text=True, stderr=subprocess.DEVNULL).strip()

        def port(service, internal):
            return int(compose("port", service, str(internal)).rsplit(":", 1)[1])

        try:
            compose("up", "-d")
            http_port, https_port = port("gateway", 80), port("gateway", 443)
            local_port = port("coordinator", 8088)
            ca_path = output / "root.crt"
            for _ in range(60):
                try:
                    ca_path.write_text(compose("exec", "-T", "gateway", "cat", "/data/caddy/pki/authorities/local/root.crt"))
                    break
                except subprocess.CalledProcessError:
                    time.sleep(1)
            context = ssl.create_default_context(cafile=str(ca_path))

            def request(path, body=None, domain=host, headers=None, https=True):
                connection = http.client.HTTPConnection("127.0.0.1", https_port if https else http_port, timeout=12)
                if https:
                    connection.sock = context.wrap_socket(socket.create_connection(("127.0.0.1", https_port), timeout=12), server_hostname=domain)
                values = {"Host": domain, "Content-Type": "application/json", "X-CloudLab-Client": "web", **(headers or {})}
                connection.request("POST" if body is not None else "GET", path,
                                   None if body is None else json.dumps(body).encode(), values)
                response = connection.getresponse()
                result = response.status, dict(response.getheaders()), response.read()
                connection.close()
                return result

            status, headers, _ = request("/", domain=plan["ip"], https=False)
            assert status == 308 and headers["Location"] == plan["dashboard"]
            assert request("/", domain="unknown.invalid", https=False)[0] == 404
            health = json.loads(request("/api/health")[2])
            assert health["status"] == "ok" and len(health["instance_id"]) == 64
            local = http.client.HTTPConnection("127.0.0.1", local_port)
            local.request("GET", "/api/health")
            assert json.loads(local.getresponse().read())["instance_id"] == health["instance_id"]
            local.close()
            assert request("/api/state")[0] == 401
            assert request("/api/tls/allow?domain=" + host)[0] == 404
            owner = compose("exec", "-T", "coordinator", "cat", "/data/admin-token")
            status, headers, _ = request("/api/login", {"token": owner}, headers={"Origin": plan["dashboard"]})
            assert status == 200 and "; Secure" in headers["Set-Cookie"] and "HttpOnly" in headers["Set-Cookie"]
            cookie = headers["Set-Cookie"].split(";")[0]

            def api(path, body=None, extra=None):
                status, _, payload = request("/api" + path, body, headers={"Cookie": cookie, **(extra or {})})
                assert status == 200, (path, status)
                return json.loads(payload)

            state = api("/state")
            assert state["remote_access"] == {"managed": True, "public_url": plan["dashboard"], "workspace_url": plan["workspaces"]}
            # A settings edit cannot expand the certificate permissions.
            connection = http.client.HTTPConnection("127.0.0.1", local_port)
            settings = {**state["settings"], "public_url": "https://unknown.invalid"}
            connection.request("PUT", "/api/settings", json.dumps(settings), {"Cookie": cookie, "X-CloudLab-Client": "web", "Content-Type": "application/json"})
            response = connection.getresponse()
            assert response.status == 400
            response.read()
            connection.close()

            lab_id = state["labs"][0]["id"]
            enrollment = api("/enrollments", {"lab_id": lab_id, "name": "Gateway test node"})
            node = api("/agent/enroll", {"token": enrollment["token"], "platform": "Linux", "arch": "aarch64", "cpus": 4, "memory_mb": 4096})
            auth = {"Authorization": "Bearer " + node["token"]}
            # Keep a real agent WebSocket open through Caddy. The fixture only
            # tests routing/authentication; it does not execute container jobs.
            relay = context.wrap_socket(socket.create_connection(("127.0.0.1", https_port), timeout=12), server_hostname=host)
            relay.sendall(("GET /api/agent/tunnel HTTP/1.1\r\nHost: " + host + "\r\n"
                           "Authorization: Bearer " + node["token"] + "\r\n"
                           "Connection: Upgrade\r\nUpgrade: websocket\r\n"
                           "Sec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n").encode())
            relay_reader = relay.makefile("rb")
            assert b"101" in relay_reader.readline()
            while relay_reader.readline() not in (b"\r\n", b""):
                pass
            heartbeat = {"cpu_usage": 1, "memory_used_mb": 100, "docker": True, "ready": True, "containers": {}}
            api("/agent/poll", heartbeat, auth)
            workspace = api("/workspaces", {"lab_id": lab_id, "node_id": node["id"], "name": "Gateway notebook", "template": "jupyter", "cpus": 1, "memory_mb": 512, "network": False})
            job = api("/agent/poll", heartbeat, auth)["job"]
            assert job["action"] == "create"
            api("/agent/jobs/" + job["id"], {"ok": True}, auth)
            app_host = "w-" + workspace["id"] + "." + plan["suffix"]
            # Caddy issues a test certificate on demand only after registration.
            assert request("/", domain=app_host)[0] == 401
            launch = urlsplit(api("/workspaces/" + workspace["id"] + "/open", {})["url"])
            status, headers, _ = request("/?" + launch.query, domain=app_host)
            assert status == 303 and "__Host-cloudlab_app=" in headers["Set-Cookie"] and "; Secure" in headers["Set-Cookie"]
            app_cookie = headers["Set-Cookie"].split(";")[0]
            status, _, metadata = request("/_cloudlab/workspace.json", domain=app_host, headers={"Cookie": app_cookie})
            assert status == 200 and json.loads(metadata)["name"] == "Gateway notebook"
            try:
                request("/", domain="w-" + str(uuid.uuid4()) + "." + plan["suffix"])
                raise AssertionError("Unknown workspace received a certificate")
            except ssl.SSLError:
                pass
            api("/logout", {})
            assert request("/_cloudlab/workspace.json", domain=app_host, headers={"Cookie": app_cookie})[0] == 401
            relay_reader.close()
            relay.close()
            print("Cloud gateway passed: IP redirect, verified HTTPS, protected login, isolated app certificates, scoped sessions, and unknown-host rejection.")
        finally:
            compose("down", "--volumes", "--remove-orphans")


if __name__ == "__main__":
    main()
