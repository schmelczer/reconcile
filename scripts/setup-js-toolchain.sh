#!/bin/bash

set -e
cd "$(dirname "$0")/.."

if ! node -e '
  const [major, minor] = process.versions.node.split(".").map(Number);
  process.exit(major === 22 && minor >= 15 ? 0 : 1);
'; then
  echo 'Error: Node.js 22.15.0 or newer in the 22.x series is required. Run `nvm install` and `nvm use`.' >&2
  exit 1
fi

wasm_pack_version=0.15.1
if ! command -v wasm-pack >/dev/null 2>&1 || [ "$(wasm-pack --version)" != "wasm-pack $wasm_pack_version" ]; then
  cargo install --locked --version "$wasm_pack_version" wasm-pack
fi
