#!/usr/bin/env python3
"""Offline checks for cloud setup; never contacts metadata, DNS, or a cloud API."""
import importlib.util
import json
import os
import tempfile
import unittest
import urllib.error
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("cloud_access", ROOT / "scripts/cloud-access.py")
cloud = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cloud)


class CloudSetupTests(unittest.TestCase):
    def test_only_public_ipv4_and_dns_suffixes_are_accepted(self):
        for value in ("127.0.0.1", "10.1.2.3", "169.254.169.254", "172.31.1.2", "192.168.1.2",
                      "100.64.1.2", "192.0.2.1", "198.51.100.4", "203.0.113.1", "224.0.0.1",
                      "0.0.0.0", "8.8.8.8:443", "https://8.8.8.8", "::1", "8.8.8.8\nexec bad"):
            with self.subTest(value=value), self.assertRaises(cloud.SetupError):
                cloud.public_ipv4(value)
        self.assertEqual(cloud.public_ipv4(" 8.8.8.8 "), "8.8.8.8")
        for value in ("localhost", "*.example.com", "https://example.com", "x\nrespond", "example.com:443", "a_b.example"):
            with self.subTest(value=value), self.assertRaises(cloud.SetupError):
                cloud.addresses("8.8.8.8", value)
        self.assertEqual(cloud.addresses("8.8.8.8", "Cloud.Example.com.")["dashboard"], "https://lab.cloud.example.com")

    def test_aws_uses_imdsv2_and_never_falls_back_to_v1(self):
        calls = []
        def fetch(path, **kwargs):
            calls.append((path, kwargs))
            return "metadata-token" if path.endswith("token") else "8.8.8.8"
        self.assertEqual(cloud.detect_ip("aws", fetch), ("aws", "8.8.8.8"))
        self.assertEqual(calls[0][1], {"method": "PUT", "headers": {"X-aws-ec2-metadata-token-ttl-seconds": "60"}})
        self.assertEqual(calls[1], ("/latest/meta-data/public-ipv4", {"headers": {"X-aws-ec2-metadata-token": "metadata-token"}}))
        calls.clear()
        def unavailable(path, **kwargs):
            calls.append(path)
            raise OSError("metadata unavailable")
        with self.assertRaises(cloud.SetupError):
            cloud.detect_ip("aws", unavailable)
        self.assertEqual(calls, ["/latest/api/token"])

    def test_other_provider_metadata_headers_and_reserved_ip(self):
        calls = []
        def fetch(path, **kwargs):
            calls.append((path, kwargs))
            return "8.8.8.8"
        self.assertEqual(cloud.detect_ip("gcp", fetch), ("gcp", "8.8.8.8"))
        self.assertEqual(calls[-1][1]["headers"], {"Metadata-Flavor": "Google"})
        cloud.detect_ip("azure", fetch)
        self.assertEqual(calls[-1][1]["headers"], {"Metadata": "true"})
        self.assertIn("format=text", calls[-1][0])
        cloud.detect_ip("digitalocean", fetch)
        self.assertEqual(calls[-1][0], "/metadata/v1/reserved_ip/ipv4/ip_address")
        def droplet(path, **kwargs):
            return "" if "reserved_ip" in path else "1.1.1.1"
        self.assertEqual(cloud.detect_ip("digitalocean", droplet), ("digitalocean", "1.1.1.1"))

    def test_auto_detection_recovers_and_manual_provider_never_probes(self):
        def fetch(path, **kwargs):
            if "computeMetadata" not in path:
                raise OSError("not this provider")
            return "8.8.8.8"
        self.assertEqual(cloud.detect_ip("auto", fetch), ("gcp", "8.8.8.8"))
        with patch.object(cloud, "metadata") as network, tempfile.TemporaryDirectory() as temporary:
            cloud.main(["--provider", "other", "--public-ip", "8.8.8.8", "--output", temporary])
            network.assert_not_called()
            self.assertEqual(json.loads((Path(temporary) / "plan.json").read_text())["ip"], "8.8.8.8")

    def test_metadata_ignores_proxies_rejects_redirects_and_bounds_reads(self):
        with patch.dict(os.environ, {"HTTP_PROXY": "http://untrusted.invalid"}), \
             patch.object(cloud.urllib.request, "build_opener") as builder:
            response = builder.return_value.open.return_value.__enter__.return_value
            response.read.return_value = b"8.8.8.8"
            self.assertEqual(cloud.metadata("/test"), "8.8.8.8")
            self.assertEqual(builder.call_args.args[0].proxies, {})
            self.assertIsInstance(builder.call_args.args[1], cloud.NoRedirect)
            response.read.assert_called_once_with(4097)
            response.read.return_value = b"x" * 4097
            with self.assertRaises(cloud.SetupError):
                cloud.metadata("/test")
        with self.assertRaises(cloud.SetupError):
            cloud.NoRedirect().redirect_request(None, None, 302, "", {}, "https://untrusted.invalid")

    def test_plan_does_not_install_and_generated_stack_keeps_internal_ports_private(self):
        with tempfile.TemporaryDirectory() as temporary, patch.object(cloud, "run") as run:
            cloud.main(["--provider", "aws", "--public-ip", "8.8.8.8", "--output", temporary])
            run.assert_not_called()
            path = Path(temporary)
            config = json.loads((path / "compose.yaml").read_text())
            services = config["services"]
            self.assertEqual(services["coordinator"]["ports"], ["127.0.0.1:8088:8088"])
            self.assertEqual(services["gateway"]["ports"], ["80:80", "443:443"])
            self.assertNotIn("docker.sock", json.dumps(config))
            self.assertTrue(services["coordinator"]["read_only"])
            self.assertEqual(services["coordinator"]["environment"]["CLOUDLAB_PUBLIC_URL"], "https://lab.8-8-8-8.sslip.io")
            caddy = (path / "Caddyfile").read_text()
            self.assertIn("ask http://coordinator:8088/api/tls/allow", caddy)
            self.assertIn("respond @internal 404", caddy)
            self.assertIn("redir https://lab.8-8-8-8.sslip.io 308", caddy)
            self.assertIn("reverse_proxy coordinator:8089", caddy)
            if os.name == "posix":
                self.assertEqual((path / "plan.json").stat().st_mode & 0o777, 0o600)
            # Rerunning replaces files without dropping data volumes.
            cloud.main(["--provider", "aws", "--public-ip", "1.1.1.1", "--output", temporary])
            updated = json.loads((path / "compose.yaml").read_text())
            self.assertEqual(config["volumes"], updated["volumes"])

    def test_service_retains_pairing_and_starts_after_reboot(self):
        service = cloud.agent_service(["docker", "render"])
        for expected in ("DynamicUser=yes", "SupplementaryGroups=docker render", "StateDirectory=cloudlab-cloud-agent",
                         "Restart=always", "WantedBy=multi-user.target", "--coordinator http://127.0.0.1:8088"):
            self.assertIn(expected, service)
        self.assertNotIn("--enrollment", service)

    def test_failed_prepare_keeps_existing_deployment_files(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            previous = cloud.addresses("8.8.8.8")
            cloud.write_plan(output, previous)
            original = (output / "compose.yaml").read_bytes()
            with patch.object(cloud, "preflight"), \
                 patch.object(cloud, "prepare", side_effect=cloud.SetupError("build failed")), \
                 patch.object(cloud, "install") as install:
                with self.assertRaisesRegex(cloud.SetupError, "build failed"):
                    cloud.main(["--provider", "aws", "--public-ip", "1.1.1.1", "--output", temporary, "--apply"])
                install.assert_not_called()
            self.assertEqual((output / "compose.yaml").read_bytes(), original)
            self.assertFalse(list(output.glob("prepare-*")))

    def test_changed_automatic_ip_does_not_silently_rotate_deployment(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            (output / "deployment.json").write_text(json.dumps(cloud.addresses("8.8.8.8")))
            with patch.object(cloud, "preflight"), \
                 patch.object(cloud, "detect_ip", return_value=("aws", "1.1.1.1")), \
                 patch.object(cloud, "prepare") as prepare:
                with self.assertRaisesRegex(cloud.SetupError, "public IP changed"):
                    cloud.main(["--provider", "aws", "--output", temporary, "--apply"])
                prepare.assert_not_called()

    def test_private_writer_replaces_symlinks_without_modifying_target(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            target = output / "unrelated"
            target.write_text("keep me")
            destination = output / "plan.json"
            destination.symlink_to(target)
            cloud.write_file(destination, "new plan")
            self.assertEqual(target.read_text(), "keep me")
            self.assertEqual(destination.read_text(), "new plan")
            self.assertFalse(destination.is_symlink())


if __name__ == "__main__":
    unittest.main()
