# `reconcile-text`: conflict-free 3-way text merging

A Rust and TypeScript library for merging conflicting text edits without manual intervention. Unlike traditional 3-way merge tools that produce conflict markers, `reconcile-text` automatically resolves conflicts by applying both sets of changes (while updating cursor positions) using an algorithm inspired by Operational Transformation.

## Try it

✨ **[Try the interactive demo](https://schmelczer.dev/reconcile)** to see it in action!

### Install it in your project

- `cargo add reconcile-text` ([reconcile-text on crates.io](https://crates.io/crates/reconcile-text))
- `npm install reconcile-text` ([reconcile-text on NPM](https://www.npmjs.com/package/reconcile-text))

## Key features

- **No conflict markers** - Clean, merged output without Git's `<<<<<<<` markers
- **Cursor tracking** - Automatically repositions cursors and selections throughout the merging process
- **Flexible tokenisation** - Word-level (default), character-level, line-level, or custom tokenisation strategies
- **Unicode support** - Full UTF-8 support with proper handling of complex scripts and grapheme clusters
- **Cross-platform** - Native Rust performance with WebAssembly bindings for JavaScript environments

## Quick start

### Rust

Install via crates.io:

```sh
cargo add reconcile-text
```

Alternatively, add `reconcile-text` to your `Cargo.toml`:

```toml
[dependencies]
reconcile-text = "0.8"
```

Then start merging:

```rust
use reconcile_text::{reconcile, BuiltinTokenizer};

// Start with the original text
let parent = "Hello world";
// Two users edit simultaneously
let left = "Hello beautiful world";  // Added "beautiful"
let right = "Hi world";              // Changed "Hello" to "Hi"

// Reconcile combines both changes
let result = reconcile(parent, &left.into(), &right.into(), &*BuiltinTokenizer::Word);
assert_eq!(result.apply().text(), "Hi beautiful world");
```

See the [merge-file example](examples/merge-file.rs) for another example, or the [library's documentation](https://docs.rs/reconcile-text/latest/reconcile_text).

### JavaScript/TypeScript

Install via NPM:

```sh
npm install reconcile-text
```

Then use it in your application:

```javascript
import { reconcile } from 'reconcile-text';

// Start with the original text
const parent = 'Hello world';
// Two users edit simultaneously
const left = 'Hello beautiful world';
const right = 'Hi world';

const result = reconcile(parent, left, right);
console.log(result.text); // "Hi beautiful world"
```

See the [example website source](examples/website/src/index.ts) for a more complex example, or the [advanced examples document](https://github.com/schmelczer/reconcile/blob/main/docs/advanced-ts.md).

## Motivation

Collaborative editing presents the challenge of merging conflicting changes when multiple users edit documents simultaneously or asynchronously whilst offline. Traditional solutions like Conflict-free Replicated Data Types (CRDTs) or Operational Transformation (OT) work well when you control the complete editing infrastructure and can capture every individual operation ([1]). However, many workflows involve users editing with various tools, for example, Obsidian users editing Markdown files with various editors ranging from Vim to VS Code.

This creates **Differential Synchronisation** scenarios ([2], [3]): we only know the final state of each document, not the sequence of operations that produced it. This is the same challenge Git addresses, but Git requires manual conflict resolution. The key insight is that while incorrect merges in source code can introduce bugs, human text is more forgiving: a slightly imperfect sentence is often preferable to conflict markers interrupting the flow.

> **Note**: Some text domains require more careful handling. Legal contracts, for instance, could have unintended meaning changes from conflicting edits that create double negations. At the same time, semantic conflicts can still arise when merging code, even in the absence of syntactic conflicts.

Differential sync is implemented by [universal-sync](https://github.com/invisible-college/universal-sync) and my Obsidian plugin [vault-link](https://github.com/schmelczer/vault-link), and it requires a merging tool that creates conflict-free results for the best user experience.

## How it works

`reconcile-text` starts off similarly to `diff3` ([4], [5]) but adds automated conflict resolution. Given a **parent** document and two modified versions (`left` and `right`), the following happens:

1. **Tokenisation** - Input texts are split into meaningful units (words, characters, etc.) for granular merging
2. **Diff computation** - Myers' algorithm calculates differences between (parent ↔ left) and (parent ↔ right)
3. **Diff optimisation** - Operations are reordered and consolidated to maximise chained changes
4. **Operational Transformation** - Edits are woven together using OT principles, preserving all modifications and updating cursors

Whilst the primary goal of `reconcile-text` isn't to implement OT, it provides an elegant way to merge Myers' diff outputs. (For a dedicated Rust OT implementation, see [operational-transform-rs](https://github.com/spebern/operational-transform-rs).) The same could be achieved with CRDTs, which many libraries implement well for text (see [Loro](https://github.com/loro-dev/loro/), [cola](https://github.com/nomad/cola), and [automerge](https://github.com/automerge/automerge)).

However, when only the end result of concurrent changes is observable, merge quality depends entirely on the quality of the underlying 2-way diffs. For instance, `move` operations cannot be supported because Myers' algorithm decomposes them into separate `insert` and `delete` operations, regardless of the merging algorithm used.

## Comparison with other approaches

### Traditional 3-way merge (diff3, Git)

Tools like `diff3` ([4]) and Git produce **conflict markers** (`<<<<<<<` / `=======` / `>>>>>>>`) when both sides modify the same region. This works for source code where a human must verify correctness, but breaks the reading flow for prose. `reconcile-text` uses the same diff3-like foundation but adds an OT-inspired resolution step that eliminates conflict markers entirely. Libraries like [diffy](https://crates.io/crates/diffy), [merge3](https://github.com/breezy-team/merge3-rs) (Rust), and [node-diff3](https://github.com/bhousel/node-diff3) (JavaScript) all fall into this category.

### diff-match-patch

[diff-match-patch](https://github.com/google/diff-match-patch) ([6]) is a widely-used library created by Neil Fraser at Google in 2006, providing character-level diffing (Myers' algorithm), fuzzy string matching (Bitap algorithm), and patch application. It powers Fraser's **Differential Synchronisation** protocol ([2]): compute a diff between two texts, apply the patch to a third text that may have drifted, and repeat until convergence. If a patch fails, the failure self-corrects in the next sync cycle.

The key differences from `reconcile-text`:

- **2-way vs 3-way** - diff-match-patch diffs two texts and applies the result as a patch. It has no concept of a common ancestor and cannot reason about "left changes" vs "right changes". `reconcile-text` performs true 3-way merging, understanding the intent behind each side's edits.

- **Character-level only** - Word-level and line-level diffs require encoding tokens as single Unicode characters before diffing ([7]). `reconcile-text` supports word, character, line, and custom tokenisation natively.

- **Patches can fail** - `patch_apply` returns a boolean array indicating success per patch; failed patches are silently dropped. In Differential Synchronisation, failures self-correct in the next cycle, but for one-shot merges edits can be lost. `reconcile-text` always produces a complete merged result.

- **No cursor tracking or change provenance** - diff-match-patch does not reposition cursors or track which side made which edit. `reconcile-text` does both automatically.

See the [comparison example](examples/compare-with-diff-match-patch.rs) for concrete cases where diff-match-patch garbles adjacent edits and silently drops an entire sentence, while `reconcile-text` merges both users' changes correctly.

> **When to use diff-match-patch instead**: when you don't have a common ancestor, for example synchronising texts that have diverged through an unknown sequence of edits. If you have a common ancestor (as in most version control and collaborative editing scenarios), `reconcile-text` produces more reliable results.

### CRDTs (Yjs, Automerge, Loro, diamond-types)

Conflict-free Replicated Data Types guarantee convergence by mathematical construction: every operation commutes, so the order of application doesn't matter. Libraries like [Yjs](https://github.com/yjs/yjs) (and its Rust port [Yrs](https://github.com/y-crdt/y-crdt)), [Automerge](https://github.com/automerge/automerge), [Loro](https://github.com/loro-dev/loro), [cola](https://github.com/nomad/cola), and [diamond-types](https://github.com/josephg/diamond-types) implement this approach.

CRDTs capture every individual keystroke or operation, assigning each a unique identity. This makes them ideal when you control the complete editing infrastructure: the editor, the transport layer, and the storage format. They work peer-to-peer, handle arbitrary numbers of concurrent editors, and never lose an edit.

The trade-off is that CRDTs require **maintaining document state over time** - an operation log or internal data structure that grows with the document's edit history. You cannot simply hand a CRDT library three plain strings and get a merged result. This makes them unsuitable for Differential Synchronisation scenarios where you only observe the final state of each document, which is exactly the niche `reconcile-text` fills.

> **When to use CRDTs instead**: if you control the complete editing stack and can capture every operation as it happens, CRDTs provide stronger convergence guarantees. They also support more than two concurrent editors naturally, whereas `reconcile-text` merges exactly two forks at a time (though merges can be chained).

### Operational Transformation (OT)

OT libraries like [ot.js](https://ot.js.org/) and [ShareJS](https://github.com/josephg/ShareJS) transform concurrent operations against each other so that applying them in any order produces the same result. Like CRDTs, they capture individual operations and require infrastructure to coordinate them, typically a central server that determines the canonical operation order.

`reconcile-text` borrows the *concept* of OT (transforming one side's edits against the other) but applies it to a different problem. Instead of transforming individual keystrokes in real time, it transforms the consolidated diff output of two complete edits. This means it doesn't need a server, doesn't need to capture operations as they happen, and works entirely offline.

> **When to use OT instead**: if you need real-time collaboration with sub-second latency and can run a coordination server, dedicated OT libraries handle this well. `reconcile-text` is designed for merge points, not live keystroke-by-keystroke synchronisation.

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
3. Optionally, set as default: 
   ```sh
   nvm alias default 22
   ```

#### Rust toolchain

Install [rustup](https://rustup.rs):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

### Scripts

- **Run tests**: `scripts/test.sh`
- **Lint and format**: `scripts/lint.sh`
- **Develop demo website**: `scripts/dev-website.sh`
- **Build demo website**: `scripts/build-website.sh`
- **Publish new version**: `scripts/bump-version.sh patch`

## License

[MIT](./LICENSE)

[1]: https://marijnhaverbeke.nl/blog/collaborative-editing-cm.html
[2]: https://neil.fraser.name/writing/sync/
[3]: https://www.cis.upenn.edu/~bcpierce/papers/diff3-short.pdf
[4]: https://blog.jcoglan.com/2017/05/08/merging-with-diff3/
[5]: https://static.googleusercontent.com/media/research.google.com/en//pubs/archive/35605.pdf
[6]: https://github.com/google/diff-match-patch
[7]: https://github.com/google/diff-match-patch/wiki/Line-or-Word-Diffs
