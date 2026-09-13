"""Offline checks: no DuckDNS account or external update is used."""
import contextlib
import importlib.util
import io
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import urllib.error
import urllib.parse

spec = importlib.util.spec_from_file_location("duckdns_update", Path(__file__).resolve().parents[1] / "scripts/duckdns-update.py")
updater = importlib.util.module_from_spec(spec)
spec.loader.exec_module(updater)
TOKEN = "00000000-0000-4000-8000-000000000000"


class Opener:
    def __init__(self, response=b"OK\n", error=None):
        self.response, self.error, self.request = response, error, None

    def open(self, request, timeout):
        self.request = request
        self.timeout = timeout
        if self.error:
            raise self.error
        return io.BytesIO(self.response)


class DuckDNSTests(unittest.TestCase):
    def test_update_only_sets_ipv4_and_uses_verified_https_endpoint(self):
        opener = Opener()
        self.assertIn("OK", updater.update("my-cloudlab", TOKEN, opener))
        url = urllib.parse.urlsplit(opener.request.full_url)
        self.assertEqual((url.scheme, url.netloc, url.path), ("https", "www.duckdns.org", "/update"))
        self.assertEqual(urllib.parse.parse_qs(url.query, keep_blank_values=True),
                         {"domains": ["my-cloudlab"], "token": [TOKEN], "ip": [""]})
        self.assertEqual(opener.timeout, 20)

    def test_rejection_and_network_errors_never_leak_token(self):
        for opener in (Opener(b"KO"), Opener(error=urllib.error.URLError("failed?token=" + TOKEN))):
            with self.assertRaises(updater.UpdateError) as error:
                updater.update("my-cloudlab", TOKEN, opener)
            self.assertNotIn(TOKEN, str(error.exception))

    def test_config_rejects_invalid_names_and_requires_private_file(self):
        with tempfile.TemporaryDirectory() as directory:
            file = Path(directory) / "duckdns.env"
            file.write_text(f"DUCKDNS_SUBDOMAIN=my-cloudlab\nDUCKDNS_API_TOKEN={TOKEN}\n")
            file.chmod(0o600)
            self.assertEqual(updater.read_config(file), ("my-cloudlab", TOKEN))
            for name in ("*.example", "lab.example.duckdns.org", "name&txt=bad", "-name"):
                file.write_text(f"DUCKDNS_SUBDOMAIN={name}\nDUCKDNS_API_TOKEN={TOKEN}\n")
                with self.assertRaises(updater.UpdateError): updater.read_config(file)
            if updater.sys.platform != "win32":
                file.chmod(0o644)
                with self.assertRaises(updater.UpdateError): updater.read_config(file)

    def test_redirects_are_not_followed(self):
        with self.assertRaises(updater.UpdateError):
            updater.NoRedirect().redirect_request(None, None, 302, "", {}, "https://elsewhere.example")

    def test_invalid_text_encoding_has_a_helpful_error(self):
        with tempfile.TemporaryDirectory() as directory:
            file = Path(directory) / "duckdns.env"
            file.write_bytes(b"\xff\xfeD\x00U\x00C\x00K\x00")
            file.chmod(0o600)
            with self.assertRaisesRegex(updater.UpdateError, "UTF-8"):
                updater.read_config(file)

    def test_failed_once_exits_nonzero_without_a_traceback(self):
        output = io.StringIO()
        with patch.object(updater, "read_config", side_effect=updater.UpdateError("Invalid config")), contextlib.redirect_stderr(output):
            self.assertEqual(updater.main([]), 1)
        self.assertIn("Invalid config", output.getvalue())
        self.assertNotIn("Traceback", output.getvalue())


if __name__ == "__main__":
    unittest.main()
