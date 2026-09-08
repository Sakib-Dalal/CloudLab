#!/usr/bin/env sh
set -eu
cd "$(dirname "$0")/.."
if [ "$#" -gt 0 ]; then
  case "$1" in terminal|jupyter|code) docker build -f "containers/$1.Dockerfile" -t "cloudlab/$1:2" . ;; *) echo 'Usage: build-images.sh [terminal|jupyter|code]' >&2; exit 1 ;; esac
else
  for template in terminal jupyter code; do
    docker build -f "containers/$template.Dockerfile" -t "cloudlab/$template:2" .
  done
fi
