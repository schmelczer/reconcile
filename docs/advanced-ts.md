# Advanced Usage (TypeScript)

## Edit Provenance

Track which changes came from where using `reconcileWithHistory`:

```javascript
const result = reconcileWithHistory(
  'Hello world',
  'Hello beautiful world',
  'Hi world'
);

console.log(result.text);    // "Hi beautiful world"
console.log(result.history); /*
[
  {
    "text": "Hello",
    "history": "RemovedFromRight"
  },
  {
    "text": "Hi",
    "history": "AddedFromRight"
  },
  {
    "text": " beautiful",
    "history": "AddedFromLeft"
  },
  {
    "text": " ",
    "history": "Unchanged"
  },
  {
    "text": "world",
    "history": "Unchanged"
  }
]
*/
```

## Tokenisation Strategies

Reconcile offers different approaches to split text for merging:

- **Word tokeniser** (`"Word"`) — Splits on word boundaries (recommended for prose)
- **Character tokeniser** (`"Character"`) — Individual characters (fine-grained control)
- **Line tokeniser** (`"Line"`) — Line-by-line (similar to `git merge` or more precisely [`git merge-file`](https://git-scm.com/docs/git-merge-file))

## Cursor Tracking

Reconcile automatically tracks cursor positions through merges, which is handy in collaborative editors. Selections can be tracked by providing them as a pair of cursors.

```javascript
const result = reconcile(
  'Hello world',
  {
    text: 'Hello beautiful world',
    cursors: [{ id: 1, position: 6 }], // After "Hello "
  },
  {
    text: 'Hi world',
    cursors: [{ id: 2, position: 0 }], // At the beginning
  }
);

// Result: "Hi beautiful world" with repositioned cursors
console.log(result.text);    // "Hi beautiful world"
console.log(result.cursors); // [{ id: 2, position: 0 }, { id: 1, position: 3 }]
```
> The `cursors` list is sorted by character position (not IDs).
