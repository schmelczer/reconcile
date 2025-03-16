#!/bin/bash

npm install -g openapi-typescript
openapi-typescript http://localhost:3030/api.json --output frontend/sync-client/src/services/types.ts
