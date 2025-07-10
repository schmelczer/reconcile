# Reconcile-text: 3-way text merging with automatic conflict resolution

A library for merging conflicting text edits without manual intervention. Unlike traditional 3-way merge tools that produce conflict markers, `reconcile-text` automatically resolves conflicts by applying both sets of changes where possible using algorithms inspired by Operational Transformation.

**[Try the interactive demo](https://schmelczer.dev/reconcile)** to see it in action.

Find it on:

- [reconcile-text on crates.io](https://crates.io/crates/reconcile-text)
- [reconcile-text on NPM](https://www.npmjs.com/package/reconcile-text)

## Key features

- **No conflict markers** — Clean, merged output without Git's `<<<<<<<` markers
- **Cursor tracking** — Automatically repositions cursors and selections during merging
- **Flexible tokenisation** — Word-level (default), character-level, or custom strategies
- **Unicode support** — Full UTF-8 support with proper handling of complex scripts
- **Cross-platform** — Native Rust performance with WebAssembly for JavaScript

## Quick start

### Rust

Run `cargo add reconcile-text` or add `reconcile-text` to your `Cargo.toml`:

```toml
[dependencies]
reconcile-text = "0.4"
```

Then merge away:

```rust
use reconcile_text::{reconcile, BuiltinTokenizer};

// Start with original text
let parent = "Hello world";
// Two people edit simultaneously
let left = "Hello beautiful world";  // Added "beautiful"
let right = "Hi world";              // Changed "Hello" to "Hi"

// Reconcile combines both changes intelligently
let result = reconcile(parent, &left.into(), &right.into(), &*BuiltinTokenizer::Word);
assert_eq!(result.apply().text(), "Hi beautiful world");
```

### JavaScript/TypeScript

Install via npm:

```bash
npm install reconcile-text
```

Then use in your application:

```javascript
import { reconcile } from 'reconcile-text';

// Same example as above
const parent = 'Hello world';
const left = 'Hello beautiful world';
const right = 'Hi world';

const result = reconcile(parent, left, right);
console.log(result.text); // "Hi beautiful world"
```

## Advanced usage

### Edit provenance

Track which changes came from where using `reconcileWithHistory`:

```javascript
const result = reconcileWithHistory(parent, left, right);
console.log(result.history); // Detailed breakdown of each text span's origin
```

### Tokenisation strategies

Reconcile offers different ways to split text for merging:

- **Word tokeniser** (`BuiltinTokenizer::Word`) — Splits on word boundaries (recommended for prose)
- **Character tokeniser** (`BuiltinTokenizer::Character`) — Individual characters (fine-grained control)
- **Line tokeniser** (`BuiltinTokenizer::Line`) — Line-by-line (similar to `git merge` or more precisely [`git merge-file`](https://git-scm.com/docs/git-merge-file))
- **Custom tokeniser** — Roll your own for specialised use cases

### Cursor tracking

Ideal for collaborative editors — Reconcile automatically tracks cursor positions through merges:

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
console.log(result.cursors); // [{ id: 1, position: 3 }, { id: 2, position: 0 }]
```

## How it works

Reconcile builds upon the foundation of `diff3` but adds intelligent conflict resolution. Given a **parent** document and two modified versions (`left` and `right`), here's what happens:

1. **Diff computation** — Myers' algorithm calculates differences between (parent ↔ left) and (parent ↔ right)
2. **Tokenisation** — Text splits into meaningful units (words, characters, etc.) for granular merging
3. **Diff optimisation** — Operations are reordered and consolidated to maximise coherent changes
4. **Operational Transformation** — Edits are woven together using OT principles, preserving all modifications

Whilst Reconcile's primary goal isn't implementing Operational Transformation, OT provides an elegant way to merge Myers' diff output. The same could be achieved with CRDTs, though the quality depends entirely on the underlying 2-way diffs. Note that `move` operations aren't supported, as Myers' algorithm decomposes them into separate `insert` and `delete` operations.

## Background

Collaborative editing presents the challenge of merging conflicting changes when multiple users edit documents simultaneously, or when synchronising edits across devices.

Traditional solutions like CRDTs or Operational Transformation work well when you control the entire editing environment and can capture every operation. However, many workflows involve users editing with different tools — for example, Obsidian users editing Markdown files with various editors from Vim to Word.

This creates **Differential Synchronisation** scenarios [¹]: you only know the final state of each document, not the sequence of operations that produced it. This is the same challenge Git addresses, but Git requires manual conflict resolution.

The key insight is that whilst incorrect merges in source code can introduce bugs, human text is more forgiving. A slightly imperfect sentence is often preferable to conflict markers interrupting the flow.

> **Note**: Some text domains require more careful handling. Legal contracts, for instance, could have unintended meaning changes from conflicting edits that create double-negations.

## Development

### Prerequisites

#### Node.js setup

1. Install [nvm](https://github.com/nvm-sh/nvm):
   ```bash
   curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
   ```
2. Install and use Node 22:
   ```bash
   nvm install 22 && nvm use 22
   ```
3. Optionally set as default: `nvm alias default 22`

#### Rust toolchain

1. Install [rustup](https://rustup.rs):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. Install additional tools:
   ```bash
   cargo install wasm-pack cargo-insta cargo-edit
   ```

### Development scripts

- **Run tests**: `scripts/test.sh`
- **Lint and format**: `scripts/lint.sh`
- **Build demo website**: `scripts/dev-website.sh`
- **Build demo website**: `scripts/build-website.sh`
- **Publish new version**: `scripts/bump-version.sh patch`

## License

[MIT](./LICENSE)

[¹]: https://static.googleusercontent.com/media/research.google.com/en//pubs/archive/35605.pdf
