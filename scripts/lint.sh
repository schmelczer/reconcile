#!/bin/bash

set -e

cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged
cargo fmt --all

echo "Success!"