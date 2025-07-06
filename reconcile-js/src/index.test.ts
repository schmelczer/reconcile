import { init, reconcile, reconcileWithHistory } from './index';
import * as fs from 'fs';

describe('reconcile', () => {
  it('tries calling functions without init', () => {
    expect(() => reconcile('Hello', 'Hello world', 'Hi world')).toThrow(/call init()/);

    expect(() => reconcileWithHistory('Hello', 'Hello world', 'Hi world')).toThrow(
      /call init()/
    );
  });

  it('call reconcile without cursors', async () => {
    await initWasm();

    expect(reconcile('Hello', 'Hello world', 'Hi world').text).toEqual('Hi world');
  });

  it('call reconcile with cursors', async () => {
    await initWasm();

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

  it('call reconcileWithHistory', async () => {
    await initWasm();

    const result = reconcileWithHistory('Hello', 'Hello world', 'Hi world');

    expect(result.text).toEqual('Hi world');
    expect(result.history.length).toBeGreaterThan(0);
  });
});

async function initWasm() {
  const wasmBin = fs.readFileSync('../pkg/reconcile_bg.wasm');
  await init({ module_or_path: wasmBin });
}
