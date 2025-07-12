# `reconcile-text`: conflict-free 3-way text merging

A Rust and TypeScript library for merging conflicting text edits without manual intervention. Unlike traditional 3-way merge tools that produce conflict markers, `reconcile-text` automatically resolves conflicts by applying both sets of changes (while updating cursor positions) using an algorithm inspired by Operational Transformation.

## Try it

✨ **[Try the interactive demo](https://schmelczer.dev/reconcile)** to see it in action!

### Install it in your project

- `cargo add reconcile-text` ([reconcile-text on crates.io](https://crates.io/crates/reconcile-text))
- `npm install reconcile-text` ([reconcile-text on NPM](https://www.npmjs.com/package/reconcile-text))

## Key features

- **No conflict markers** — Clean, merged output without Git's `<<<<<<<` markers
- **Cursor tracking** — Automatically repositions cursors (and selections) during merging
- **Flexible tokenisation** — Word-level (default), character-level, or custom strategies
- **Unicode support** — Full UTF-8 support with proper handling of complex scripts
- **Cross-platform** — Native Rust performance with WebAssembly for JavaScript

## Quick start

### Rust

Install via crates.io:
```sh
cargo add reconcile-text
```

or add `reconcile-text` to your `Cargo.toml`:

```toml
[dependencies]
reconcile-text = "0.5"
```

Then merge away:

```rust
use reconcile_text::{reconcile, BuiltinTokenizer};

// Start with original text
let parent = "Hello world";
// Two users edit simultaneously
let left = "Hello beautiful world";  // Added "beautiful"
let right = "Hi world";              // Changed "Hello" to "Hi"

// Reconcile combines both changes
let result = reconcile(parent, &left.into(), &right.into(), &*BuiltinTokenizer::Word);
assert_eq!(result.apply().text(), "Hi beautiful world");
```

See [merge-file](examples/merge-file.rs) for another example or the [library's documentation](https://docs.rs/reconcile-text/latest/reconcile_text).

### JavaScript/TypeScript

Install via NPM:

```sh
npm install reconcile-text
```

Then use it in your application:

```javascript
import { reconcile } from 'reconcile-text';

// Start with original text
const parent = 'Hello world';
// Two users edit simultaneously
const left = 'Hello beautiful world';
const right = 'Hi world';

const result = reconcile(parent, left, right);
console.log(result.text); // "Hi beautiful world"
```

See the [example website](examples/website/src/index.ts) for a more complex example or the [advanced examples document](https://github.com/schmelczer/reconcile/blob/main/docs/advanced-ts.md).

## Motivation

Collaborative editing presents the challenge of merging conflicting changes when multiple users edit documents simultaneously (or offline). Traditional solutions like Conflict-free Replicated Data Types (CRDTs) or Operational Transformation (OT) works well when you control the entire editing environment  and can capture every operation ([1]). However, many workflows involve users editing with different tools — for example, Obsidian users editing Markdown files with various editors from Vim to VS Code.

This creates **Differential Synchronisation** scenarios ([2], [3]): we only know the final state of each document, not the sequence of operations that produced it. This is the same challenge Git addresses, but Git requires manual conflict resolution. The key insight is that while incorrect merges in source code can introduce bugs, human text is more forgiving. A slightly imperfect sentence is often preferable to conflict markers interrupting the flow.

> **Note**: Some text domains require more careful handling. Legal contracts, for instance, could have unintended meaning changes from conflicting edits that create double-negations. At the same time, semantic conflicts can still arise when merging code, even in the absence of syntactical conflicts.

Differenctial sync is implemented by [universal-sync](https://github.com/invisible-college/universal-sync) and my Obsidian plugin, [vault-link](https://github.com/schmelczer/vault-link) and it requires a merging tool which creates conflict free results for the best user experience.

## How it works

`reconcile-text` starts off similarly to `diff3` ([4], [5]) but adds automated conflict resolution. Given a **parent** document and two modified versions (`left` and `right`), the following happens:

1. **Tokenisation** — Input texts get split into meaningful units (words, characters, etc.) for granular merging
2. **Diff computation** — Myers' algorithm calculates differences between (parent ↔ left) and (parent ↔ right)
3. **Diff optimisation** — Operations are reordered and consolidated to maximise chained changes
4. **Operational Transformation** — Edits are woven together using OT principles, preserving all modifications and updating cursors

While the primary goal of `reconcile-text` isn't to implement OT (you can check out [operational-transform-rs](https://github.com/spebern/operational-transform-rs) for a Rust implementation of it), OT provides an elegant way to merge Myers' diff outputs. The same could be achieved with CRDTs which many libraries implement well for text: see [Loro](https://github.com/loro-dev/loro/), [cola](https://github.com/nomad/cola), and [automerge](https://github.com/automerge/automerge) as a few great examples. 

However, the quality of a merge, if only the end result of concurrent changes is observable, depends entirely on the quality of the underlying 2-way diffs. For instance, `move` operations can't be supported as Myers' algorithm decomposes them into separate `insert` and `delete` operations regardless the merging algorithm.

## Development

Contributions are welcome!

### Environment

#### Node.js setup

1. Install [nvm](https://github.com/nvm-sh/nvm):
   ```sh
   curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
   ```
2. Install and use Node 22:
   ```sh
   nvm install 22 && nvm use 22
   ```
3. Optionally set as default: 
   ```sh
   nvm alias default 22
   ```

#### Rust toolchain

1. Install [rustup](https://rustup.rs):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. Install additional tools:
   ```bash
   cargo install wasm-pack cargo-insta cargo-edit
   ```

### Scripts

- **Run tests**: `scripts/test.sh`
- **Lint and format**: `scripts/lint.sh`
- **Build demo website**: `scripts/dev-website.sh`
- **Build demo website**: `scripts/build-website.sh`
- **Publish new version**: `scripts/bump-version.sh patch`

## License

[MIT](./LICENSE)

[1]:https://marijnhaverbeke.nl/blog/collaborative-editing-cm.html
[2]: https://neil.fraser.name/writing/sync/ 
[3]: https://www.cis.upenn.edu/~bcpierce/papers/diff3-short.pdf
[4]: https://blog.jcoglan.com/2017/05/08/merging-with-diff3/
[5]: https://static.googleusercontent.com/media/research.google.com/en//pubs/archive/35605.pdf
