#!/bin/bash

set -e
cd "$(dirname "$0")/.."

npm run build --prefix examples/website
