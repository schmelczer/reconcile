import { reconcile, reconcileWithHistory, diff, undiff } from './index';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

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

describe('test_merge_files_without_panic', () => {
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
