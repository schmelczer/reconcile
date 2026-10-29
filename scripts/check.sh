#!/bin/bash

set -e
cd "$(dirname "$0")/.."

scripts/setup.sh
scripts/build.sh
scripts/lint.sh
scripts/test.sh
scripts/build-website.sh
