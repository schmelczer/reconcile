import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'node',
    alias: {
      'reconcile-text/reconcile_text_bg.wasm': fileURLToPath(
        new URL('./__mocks__/wasm.mjs', import.meta.url)
      ),
    },
  },
});
