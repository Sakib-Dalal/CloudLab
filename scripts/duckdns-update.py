#!/usr/bin/env python3
"""Update one DuckDNS IPv4 address over verified HTTPS; optionally every 5 minutes.

No third-party packages. Credentials stay in a private config file, never in
command-line arguments or output. This updater does not touch ACME TXT records.
"""
import argparse
from pathlib import Path
import re
import stat
import sys
import time
import urllib.error
import urllib.parse
import urllib.request


class UpdateError(Exception):
    pass


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        # Never forward the account token to a redirect destination.
        raise UpdateError("DuckDNS redirected the update. Try again later.")


def read_config(path):
    try:
        if sys.platform != "win32" and stat.S_IMODE(path.stat().st_mode) & 0o077:
            raise UpdateError("Make the config private first: chmod 600 CONFIG_FILE")
        values = {}
        for line in path.read_text(encoding="utf-8-sig").splitlines():
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            key, separator, value = line.partition("=")
            if not separator or key not in ("DUCKDNS_SUBDOMAIN", "DUCKDNS_API_TOKEN") or key in values:
                raise UpdateError("Use the two KEY=VALUE lines from deploy/duckdns.env.example.")
            values[key] = value.strip()
    except OSError:
        raise UpdateError("Cannot read the DuckDNS config file. Check its path and permissions.") from None
    except UnicodeError:
        raise UpdateError("Save the DuckDNS config as a UTF-8 text file using the supplied example.") from None
    domain = values.get("DUCKDNS_SUBDOMAIN", "")
    token = values.get("DUCKDNS_API_TOKEN", "")
    if not re.fullmatch(r"[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?", domain):
        raise UpdateError("Use your registered DuckDNS name only, without dots or a wildcard.")
    if not re.fullmatch(r"[0-9a-fA-F]{8}(?:-[0-9a-fA-F]{4}){3}-[0-9a-fA-F]{12}", token):
        raise UpdateError("Replace the placeholder with your DuckDNS account token in the private config file.")
    return domain, token


def update(domain, token, opener=None):
    # Blank ip asks DuckDNS to detect this gateway's public IPv4 address.
    query = urllib.parse.urlencode({"domains": domain, "token": token, "ip": ""})
    request = urllib.request.Request("https://www.duckdns.org/update?" + query,
                                     headers={"User-Agent": "CloudLab-DuckDNS/2.0"})
    opener = opener or urllib.request.build_opener(NoRedirect())
    try:
        with opener.open(request, timeout=20) as response:
            result = response.read(128).decode("ascii", errors="replace").strip()
    except (OSError, urllib.error.URLError):
        # urllib exceptions may contain the credential-bearing URL.
        raise UpdateError("Could not reach DuckDNS over HTTPS. Check this gateway's internet connection.") from None
    if result != "OK":
        raise UpdateError("DuckDNS rejected the update. Check the account token and registered name.")
    return "OK: DuckDNS IPv4 address updated."


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", type=Path, default=Path(".cloudlab/duckdns.env"))
    parser.add_argument("--watch", action="store_true", help="Update now and every 300 seconds until stopped")
    args = parser.parse_args(argv)
    try:
        while True:
            try:
                # Re-read on each cycle so token changes do not require a restart.
                print(update(*read_config(args.config)), flush=True)
            except UpdateError as error:
                print(f"DuckDNS: {error}", file=sys.stderr, flush=True)
                if not args.watch:
                    return 1
            if not args.watch:
                return 0
            time.sleep(300)
    except KeyboardInterrupt:
        return 0


if __name__ == "__main__":
    sys.exit(main())
