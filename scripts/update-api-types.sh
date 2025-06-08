#!/bin/bash

set -e

rm -rf backend/sync_server/bindings

cd backend
cargo test export_bindings
cd -

cp -r backend/sync_server/bindings/* frontend/sync-client/src/services/types/
