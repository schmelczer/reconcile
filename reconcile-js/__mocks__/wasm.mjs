import { readFileSync } from 'node:fs';

// Match webpack's inline WASM loader using the actual compiled binary.
export default readFileSync(
  new URL('../../pkg/reconcile_text_bg.wasm', import.meta.url)
).toString('base64');
