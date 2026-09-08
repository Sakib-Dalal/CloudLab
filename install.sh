#!/usr/bin/env sh
set -eu
cd "$(dirname "$0")"
./build.sh
task_prefix="${CLOUDLAB_INSTALL_PREFIX:-$HOME/.local}"
mkdir -p "$task_prefix/bin" "$task_prefix/share/cloudlab/web"
install -m 755 target/release/cloudlab "$task_prefix/bin/cloudlab"
cp -R dist/. "$task_prefix/share/cloudlab/web/"
echo "Installed CloudLab to $task_prefix/bin/cloudlab"
echo "Run: cloudlab serve --web-dir '$task_prefix/share/cloudlab/web'"
