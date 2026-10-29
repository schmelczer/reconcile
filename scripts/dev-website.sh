#!/bin/bash

set -e
cd "$(dirname "$0")/.."

scripts/build-js.sh
npm start --prefix examples/website
