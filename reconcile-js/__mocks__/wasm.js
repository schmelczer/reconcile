const fs = require('fs');
const path = require('path');

// Read the actual WASM file and convert to base64 for testing
const wasmPath = path.join(__dirname, '../../pkg/reconcile_bg.wasm');
const wasmBuffer = fs.readFileSync(wasmPath);
const wasmBase64 = wasmBuffer.toString('base64');

module.exports = wasmBase64;
