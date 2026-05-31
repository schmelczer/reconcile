import * as webApi from './index';
import * as rnApi from './index.rn';
import { installWasmLeakDetector, checkForWasmLeaks } from './wasm-leak-detector';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

installWasmLeakDetector();

afterEach(() => {
  const leaks = checkForWasmLeaks();
  if (leaks.length > 0) {
    throw new Error(
      `WASM memory leak: ${leaks.length} object(s) not freed:\n  ${leaks.join('\n  ')}`
    );
  }
});

// `./index` is the web/node build (real WebAssembly); `./index.rn` is the React
// Native build (the wasm2js pure-JS translation). Both are thin backends over the
// same `src/core.ts` wrapper and expose an identical public API, so the behavioural
// suite below runs against both to guarantee they stay in lock-step.
const backends = [
  { name: 'web/node (WebAssembly)', api: webApi },
  { name: 'React Native (wasm2js)', api: rnApi },
];

describe.each(backends)('reconcile [$name]', ({ api }) => {
  const { reconcile, reconcileWithHistory, diff, undiff } = api;

  it('call reconcile without cursors', () => {
    expect(reconcile('Hello', 'Hello world', 'Hi world').text).toEqual('Hi world');
  });

  it('call reconcile with cursors', () => {
    const result = reconcile(
      'Hello',
      {
        text: 'Hello world',
        cursors: [
          {
            id: 3,
            position: 2,
          },
        ],
      },
      {
        text: 'Hi world',
        cursors: [
          {
            id: 4,
            position: 0,
          },
          { id: 5, position: 3 },
        ],
      }
    );

    expect(result.text).toEqual('Hi world');
    expect(result.cursors).toEqual([
      { id: 3, position: 0 },
      { id: 4, position: 0 },
      { id: 5, position: 3 },
    ]);
  });

  it('call reconcileWithHistory', () => {
    const result = reconcileWithHistory('Hello', 'Hello world', 'Hi world');

    expect(result.text).toEqual('Hi world');
    expect(result.history.length).toBeGreaterThan(0);
  });

  it('undiff accepts bigint entries (per the Array<number | bigint | string> type)', () => {
    const original = 'Hello world';
    const changed = 'Hello cruel world';

    // `diff` returns plain numbers; emulate a caller that supplies BigInt, which the
    // public signature permits. The wasm2js build rejects raw BigInt, so the shared
    // wrapper must normalise it — running this on both backends asserts the contract.
    const withBigints = diff(original, changed).map((item) =>
      typeof item === 'number' ? BigInt(item) : item
    );

    expect(withBigints.some((item) => typeof item === 'bigint')).toBe(true);
    expect(undiff(original, withBigints)).toEqual(changed);
  });
});

describe.each(backends)('diff and undiff are inverse [$name]', ({ api }) => {
  const { diff, undiff } = api;

  const resourcesPath = path.join(__dirname, '../../tests/resources');

  const readFileSlice = (fileName: string, start: number, end: number): string => {
    const filePath = path.join(resourcesPath, fileName);
    const content = fs.readFileSync(filePath, 'utf-8');
    const chars = Array.from(content); // Handle unicode properly
    return chars.slice(start, Math.min(end, chars.length)).join('');
  };

  const files = ['pride_and_prejudice.txt', 'room_with_a_view.txt', 'blns.txt'];

  const ranges = [{ start: 0, end: 50000 }];

  files.forEach((file1) => {
    files.forEach((file2) => {
      ranges.forEach((range1) => {
        ranges.forEach((range2) => {
          it(`should diff & undiff ${file1}[${range1.start}..${range1.end}], ${file2}[${range2.start}..${range2.end}] without panic`, () => {
            const content1 = readFileSlice(file1, range1.start, range1.end);
            const content2 = readFileSlice(file2, range2.start, range2.end);

            const changes = diff(content1, content2);
            const actual = undiff(content1, changes);
            expect(actual).toEqual(content2);
          });
        });
      });
    });
  });
});

// React-Native-only: Hermes exposes no `WebAssembly` global, which is the whole reason
// the RN entry point links a wasm2js build. Only the wasm2js backend can satisfy this.
describe('React Native (wasm2js) Hermes parity', () => {
  const { reconcile, reconcileWithHistory, diff, undiff } = rnApi;

  it('runs every operation with no WebAssembly global', () => {
    const descriptor = Object.getOwnPropertyDescriptor(globalThis, 'WebAssembly');
    delete (globalThis as { WebAssembly?: unknown }).WebAssembly;
    try {
      expect((globalThis as { WebAssembly?: unknown }).WebAssembly).toBeUndefined();

      expect(reconcile('Hello', 'Hello world', 'Hi world').text).toEqual('Hi world');

      const changes = diff('Hello world', 'Hello cruel world');
      expect(undiff('Hello world', changes)).toEqual('Hello cruel world');

      expect(
        reconcileWithHistory('Hello', 'Hello world', 'Hi world').history.length
      ).toBeGreaterThan(0);
    } finally {
      // Restore the global so the leak check and later suites are unaffected.
      if (descriptor) {
        Object.defineProperty(globalThis, 'WebAssembly', descriptor);
      }
    }
  });
});
