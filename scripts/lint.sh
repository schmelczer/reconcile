#!/bin/bash

set -e
cd "$(dirname "$0")/.."

cargo machete
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check

npm run format:check --prefix reconcile-js
npm run format:check --prefix examples/website

uv run --directory reconcile-python --no-sync ruff check python/ tests/
uv run --directory reconcile-python --no-sync ruff format --check python/ tests/
uv run --directory reconcile-python --no-sync pyright python/ tests/
