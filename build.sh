#!/usr/bin/env sh
set -eu
cd "$(dirname "$0")"
npm ci
npm run check
npm run build
cargo build --release --locked -p cloudlab
echo 'Built target/release/cloudlab and dist/.'
echo 'Start with: ./target/release/cloudlab serve'
