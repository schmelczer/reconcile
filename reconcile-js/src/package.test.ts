import { execFileSync } from 'node:child_process';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { afterAll, beforeAll, test } from 'vitest';

const packageDir = resolve(dirname(fileURLToPath(import.meta.url)), '..');
let consumerDir: string;

beforeAll(() => {
  // Install the published files in isolation: this project's dependency with
  // the same name is the raw wasm-bindgen package, not the public wrapper.
  consumerDir = mkdtempSync(resolve(tmpdir(), 'reconcile-text-package-'));
  const [{ filename }] = JSON.parse(
    execFileSync(
      'npm',
      ['pack', '--json', '--ignore-scripts', '--pack-destination', consumerDir],
      { cwd: packageDir, encoding: 'utf8' }
    )
  );
  execFileSync(
    'npm',
    [
      'install',
      '--ignore-scripts',
      '--no-audit',
      '--no-fund',
      resolve(consumerDir, filename),
    ],
    { cwd: consumerDir, stdio: 'pipe' }
  );
}, 30_000);

afterAll(() => {
  if (consumerDir) {
    rmSync(consumerDir, { recursive: true, force: true });
  }
});

test.each([
  {
    name: 'ESM named imports',
    inputType: 'module',
    load: `import { reconcile, diff, undiff, reconcileWithHistory } from 'reconcile-text';`,
  },
  {
    name: 'CommonJS require',
    inputType: 'commonjs',
    load: `const { reconcile, diff, undiff, reconcileWithHistory } = require('reconcile-text');`,
  },
])('packed package supports $name', ({ inputType, load }) => {
  execFileSync(
    process.execPath,
    [
      `--input-type=${inputType}`,
      '--eval',
      `${load}
      const parent = 'Hello world';
      const left = 'Hello beautiful world';
      const right = 'Hi world';
      const expected = 'Hi beautiful world';

      if (reconcile(parent, left, right).text !== expected) {
        throw new Error('Quick start returned the wrong merged text');
      }
      if (reconcileWithHistory(parent, left, right).text !== expected) {
        throw new Error('Merge with history returned the wrong merged text');
      }
      if (undiff(parent, diff(parent, left)) !== left) {
        throw new Error('Diff/undiff did not round-trip');
      }
      `,
    ],
    { cwd: consumerDir, stdio: 'pipe' }
  );
});
