import * as webApi from './index';
import * as rnApi from './index.rn';
import * as webBindings from 'reconcile-text';
import * as rnBindings from '../pkg-rn/reconcile_text.js';
import { jest } from '@jest/globals';
import type { TextWithOptionalCursors } from './core';
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
  { name: 'web/node (WebAssembly)', api: webApi, bindings: webBindings },
  { name: 'React Native (wasm2js)', api: rnApi, bindings: rnBindings },
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

describe.each(backends)('WASM cleanup after exceptions [$name]', ({ api, bindings }) => {
  const mergeOperations = ['reconcile', 'reconcileWithHistory'] as const;
  const operations = [...mergeOperations, 'diff'] as const;
  const left = {
    text: 'Hello world',
    cursors: [{ id: 1, position: 2 }],
  };

  function failingString(error: Error): string {
    return {
      get length() {
        throw error;
      },
      [Symbol.iterator]() {
        return 'Hello world'[Symbol.iterator]();
      },
    } as unknown as string;
  }

  it.each(mergeOperations)(
    '%s frees the left input if reading the right input throws',
    (operation) => {
      const error = new Error('right input failed');
      const right = {
        get text(): string {
          throw error;
        },
        cursors: [],
      };

      expect(() => api[operation]('Hello', left, right)).toThrow(error);
    }
  );

  it.each(operations)(
    '%s frees cursors when a later input cursor throws',
    (operation) => {
      const error = new Error('cursor input failed');
      const input = {
        text: 'Hello world',
        cursors: [
          { id: 2, position: 1 },
          {
            get id(): number {
              throw error;
            },
            position: 2,
          },
        ],
      };
      const invoke = () =>
        operation === 'diff'
          ? api.diff('Hello', input)
          : api[operation]('Hello', left, input);

      expect(invoke).toThrow(error);
    }
  );

  it.each(operations)(
    '%s frees cursors if the text constructor throws before consuming them',
    (operation) => {
      const error = new Error('text input failed');
      const input: TextWithOptionalCursors = {
        text: failingString(error),
        cursors: [{ id: 2, position: 1 }],
      };
      const invoke = () =>
        operation === 'diff'
          ? api.diff('Hello', input)
          : api[operation]('Hello', left, input);

      expect(invoke).toThrow(error);
    }
  );

  it.each(operations)('%s frees inputs when the backend throws', (operation) => {
    const error = new Error('backend failed');
    const original = failingString(error);
    const invoke = () =>
      operation === 'diff'
        ? api.diff(original, left)
        : api[operation](original, left, 'Hi world');

    expect(invoke).toThrow(error);
  });

  describe.each(operations)('%s cursor validation', (operation) => {
    it.each([-1, 0.5, NaN, Infinity, 4])(
      'rejects position %s before the WASM constructor can trap',
      (position) => {
        const input = {
          text: 'a😀b',
          cursors: [
            { id: 2, position: 1 },
            { id: 3, position },
          ],
        };
        const invoke = () =>
          operation === 'diff'
            ? api.diff('Hello', input)
            : api[operation]('Hello', left, input);

        expect(invoke).toThrow(RangeError);
        expect(invoke).toThrow(
          `Cursor position ${position} must be an integer between 0 and 3`
        );
      }
    );

    it('accepts the start and Unicode scalar end positions', () => {
      const input = {
        text: 'a😀b',
        cursors: [
          { id: 2, position: 0 },
          { id: 3, position: 3 },
        ],
      };
      if (operation === 'diff') {
        expect(api.undiff('Hello', api.diff('Hello', input))).toBe(input.text);
      } else {
        expect(api[operation]('Hello', 'Hello', input).cursors).toEqual(input.cursors);
      }
    });
  });

  describe.each(mergeOperations)('%s result conversion', (operation) => {
    it.each(['id', 'characterIndex'] as const)(
      'frees all result cursors if %s throws',
      (method) => {
        const error = new Error('cursor conversion failed');
        const spy = jest
          .spyOn(bindings.CursorPosition.prototype, method)
          .mockImplementation(() => {
            throw error;
          });
        try {
          expect(() =>
            api[operation]('Hello', left, {
              text: 'Hi world',
              cursors: [{ id: 2, position: 1 }],
            })
          ).toThrow(error);
        } finally {
          spy.mockRestore();
        }
      }
    );

    it.each(['text', 'cursors'] as const)('frees the result if %s throws', (method) => {
      const error = new Error('result conversion failed');
      const prototype =
        operation === 'reconcile'
          ? bindings.TextWithCursors.prototype
          : bindings.TextWithCursorsAndHistory.prototype;
      const spy = jest.spyOn(prototype, method).mockImplementation(() => {
        throw error;
      });
      try {
        expect(() => api[operation]('Hello', left, 'Hi world')).toThrow(error);
      } finally {
        spy.mockRestore();
      }
    });
  });

  it.each(['text', 'history'] as const)(
    'frees all history spans if %s throws',
    (method) => {
      const error = new Error('history conversion failed');
      const spy = jest
        .spyOn(bindings.SpanWithHistory.prototype, method)
        .mockImplementation(() => {
          throw error;
        });
      try {
        expect(() => api.reconcileWithHistory('Hello', left, 'Hi world')).toThrow(error);
      } finally {
        spy.mockRestore();
      }
    }
  );

  it('frees the result if reading history throws', () => {
    const error = new Error('reading history failed');
    const spy = jest
      .spyOn(bindings.TextWithCursorsAndHistory.prototype, 'history')
      .mockImplementation(() => {
        throw error;
      });
    try {
      expect(() => api.reconcileWithHistory('Hello', left, 'Hi world')).toThrow(error);
    } finally {
      spy.mockRestore();
    }
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
