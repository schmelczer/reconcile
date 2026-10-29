#!/bin/bash

set -e
cd "$(dirname "$0")/.."

scripts/build-js.sh
uv run --directory reconcile-python --no-sync maturin develop -q
