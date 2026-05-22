# Advanced Usage (TypeScript)

## Edit Provenance

Track which changes came from where using `reconcileWithHistory`. The result's
`history` field is typed as `SpanWithHistory[]`, and each span's `history` is a
`History` string-literal union.

```typescript
import { reconcileWithHistory, type History, type SpanWithHistory } from 'reconcile-text';

const result = reconcileWithHistory('Hello world', 'Hello beautiful world', 'Hi world');

console.log(result.text); // "Hi beautiful world"

const history: SpanWithHistory[] = result.history;
console.log(history);
// [
//   { text: "Hello", history: "RemovedFromRight" },
//   { text: "Hi", history: "AddedFromRight" },
//   { text: " beautiful", history: "AddedFromLeft" },
//   { text: " ", history: "Unchanged" },
//   { text: "world", history: "Unchanged" },
// ]

const classByHistory = {
  Unchanged: 'merge-unchanged',
  AddedFromLeft: 'merge-added-left',
  AddedFromRight: 'merge-added-right',
  RemovedFromLeft: 'merge-removed-left',
  RemovedFromRight: 'merge-removed-right',
} satisfies Record<History, string>;
```

Using `satisfies Record<History, string>` keeps the object literal's values
narrow while forcing every history case to be handled. If a future version adds
another `History` value, TypeScript will point at this mapping.

For control flow, use the same union as an exhaustiveness check:

```typescript
import type { History } from 'reconcile-text';

function historyLabel(history: History): string {
  switch (history) {
    case 'Unchanged':
      return 'unchanged';
    case 'AddedFromLeft':
      return 'added by left';
    case 'AddedFromRight':
      return 'added by right';
    case 'RemovedFromLeft':
      return 'removed from left';
    case 'RemovedFromRight':
      return 'removed from right';
    default:
      return assertNever(history);
  }
}

function assertNever(value: never): never {
  throw new Error(`Unhandled history value: ${value}`);
}
```

## Tokenisation Strategies

`reconcile-text` offers different approaches to split text for merging:

- **Word tokeniser** (`"Word"`) - Splits on word boundaries (recommended for prose)
- **Character tokeniser** (`"Character"`) - Individual characters (fine-grained control)
- **Line tokeniser** (`"Line"`) - Line-by-line (similar to `git merge` or more precisely [`git merge-file`](https://git-scm.com/docs/git-merge-file))
- **Markdown tokeniser** (`"Markdown"`) - Splits on Markdown structural boundaries (headings, list items, paragraphs)

```typescript
import { reconcile, type BuiltinTokenizer } from 'reconcile-text';

const tokenizers = [
  'Word',
  'Character',
  'Line',
  'Markdown',
] as const satisfies readonly BuiltinTokenizer[];

const result = reconcile('abc', 'axc', 'abyc', 'Character');
console.log(result.text); // "axyc"

for (const tokenizer of tokenizers) {
  const merged = reconcile(
    '# Title\n\n- old item\n',
    '# Title\n\n- old item\n- left item\n',
    '# New title\n\n- old item\n',
    tokenizer
  );

  console.log(tokenizer, merged.text);
}
```

## Cursor Tracking

`reconcile-text` automatically tracks cursor positions through merges, which is
useful for collaborative editors. Selections can be tracked by providing them as
a pair of cursors.

```typescript
import { reconcile, type TextWithOptionalCursors } from 'reconcile-text';

const left = {
  text: 'Hello beautiful world',
  cursors: [{ id: 1, position: 6 }], // After "Hello "
} satisfies TextWithOptionalCursors;

const right = {
  text: 'Hi world',
  cursors: [{ id: 2, position: 0 }], // At the beginning
} satisfies TextWithOptionalCursors;

const result = reconcile('Hello world', left, right);

// Result: "Hi beautiful world" with repositioned cursors
console.log(result.text); // "Hi beautiful world"
console.log(result.cursors); // [{ id: 2, position: 0 }, { id: 1, position: 3 }]
```

> The `cursors` list is sorted by character position (not IDs).

## Generic Helpers and Inference

The exported merge functions are intentionally small: they merge strings, or
strings plus cursor metadata. In TypeScript applications, keep domain-specific
metadata in your own typed wrappers and let inference preserve the surrounding
shape.

```typescript
import { reconcile, type BuiltinTokenizer } from 'reconcile-text';

type ReconciledText<T extends { text: string }> = Omit<T, 'text'> & {
  text: string;
};

function reconcileDraft<TDraft extends { text: string }>(
  parent: TDraft,
  left: TDraft,
  right: TDraft,
  tokenizer?: BuiltinTokenizer
): ReconciledText<TDraft> {
  return {
    ...right,
    text: reconcile(parent.text, left.text, right.text, tokenizer).text,
  };
}

interface MarkdownDraft {
  id: string;
  text: string;
  updatedAt: Date;
}

const parent: MarkdownDraft = {
  id: 'intro',
  text: '# Title\n\nOld text\n',
  updatedAt: new Date('2026-01-01T00:00:00Z'),
};

const left: MarkdownDraft = {
  ...parent,
  text: '# Title\n\nOld text\n\n- left note\n',
};

const right: MarkdownDraft = {
  ...parent,
  text: '# New title\n\nOld text\n',
};

const merged = reconcileDraft(parent, left, right, 'Markdown');
// merged is inferred as { id: string; updatedAt: Date; text: string }
```

Use `satisfies` for configuration objects and cursor payloads when you want
compile-time checking without widening everything to the library interface.

```typescript
import type { BuiltinTokenizer, TextWithOptionalCursors } from 'reconcile-text';

const mergeOptions = {
  tokenizer: 'Markdown',
  renderDeletedSpans: true,
} satisfies {
  tokenizer: BuiltinTokenizer;
  renderDeletedSpans: boolean;
};

const documentWithSelection = {
  text: 'Hello beautiful world',
  cursors: [
    { id: 1, position: 6 },
    { id: 2, position: 15 },
  ],
} satisfies TextWithOptionalCursors;
```

## Compact Diffs

Generate and apply compact diff representations. The TypeScript type is
`Array<number | string>` for `diff()` and `Array<number | bigint | string>` for
`undiff()`, because the underlying WebAssembly layer may represent integer
entries as `bigint`.

```typescript
import { diff, undiff } from 'reconcile-text';

const original = 'Hello world';
const changed = 'Hello beautiful world';

// Generate a compact diff
const changes = diff(original, changed);
console.log(changes); // [5, " beautiful world"]

// Reconstruct the changed text from the diff
const reconstructed = undiff(original, changes);
console.assert(reconstructed === changed);
```

Diff entries are positive integers (retain N characters), negative integers
(delete N characters), and strings (insert text).

## Complete Example

For a complete browser example that renders `SpanWithHistory` values and cursor
selections, see the [example website source](../examples/website/src/index.ts).
