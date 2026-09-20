#!/usr/bin/env python3
"""Bounded, structured package operations in the workspace's persistent venv."""
import importlib.metadata
import json
import os
import re
import subprocess
import sys
import sysconfig

ENVIRONMENT = "/home/lab/.venv"
PYTHON = ENVIRONMENT + "/bin/python"

def package_name(value):
    if not isinstance(value, str) or not re.fullmatch(r"[A-Za-z0-9](?:[A-Za-z0-9._-]{0,126}[A-Za-z0-9])?", value):
        raise ValueError("Enter a package name from PyPI, without URLs or command options.")
    return re.sub(r"[-_.]+", "-", value).lower()

def main():
    if len(sys.argv) != 2:
        raise ValueError("Expected one package operation.")
    request = json.loads(sys.argv[1])
    action = request.get("action")
    if action not in ("list", "install", "remove"):
        raise ValueError("Unsupported package operation.")
    if action == "list":
        rows = {}
        local = sysconfig.get_path("purelib")
        for distribution in importlib.metadata.distributions():
            name = distribution.metadata.get("Name")
            if name:
                normalized = package_name(name)
                removable = str(distribution.locate_file("")).startswith(ENVIRONMENT + "/")
                if normalized not in rows or removable:
                    rows[normalized] = {"name": name, "version": distribution.version, "removable": removable}
        print(json.dumps({"packages": sorted(rows.values(), key=lambda p: p["name"].lower()), "environment": ENVIRONMENT, "python": sys.version.split()[0], "manager": "uv"}))
        return
    name = package_name(request.get("name"))
    version = request.get("version", "")
    if not isinstance(version, str) or (version and not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9.!+_-]{0,79}", version)):
        raise ValueError("Choose a specific package version, without options or URLs.")
    if action == "remove":
        local_names = {package_name(d.metadata["Name"]) for d in importlib.metadata.distributions(path=[sysconfig.get_path("purelib"), sysconfig.get_path("platlib")]) if d.metadata.get("Name")}
        if name not in local_names:
            raise ValueError("This package belongs to the workspace image and cannot be removed here.")
    command = ["/usr/local/bin/uv", "--no-config", "pip", "install" if action == "install" else "uninstall", "--python", PYTHON]
    if action == "install":
        command += ["--index-url", "https://pypi.org/simple", "--", name + ("==" + version if version else "")]
    else:
        command += ["--", name]
    subprocess.run(command, check=True, timeout=600)

if __name__ == "__main__":
    try:
        main()
    except (ValueError, subprocess.SubprocessError) as error:
        print(str(error), file=sys.stderr)
        sys.exit(1)
