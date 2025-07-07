import { reconcile, reconcileWithHistory } from './index';

describe('reconcile', () => {
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
});
