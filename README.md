# `reconcile-text`: conflict-free 3-way text merging

Merge a common parent and two edited versions into one text, without conflict markers. `reconcile-text` combines changes from both sides and updates any cursor positions you pass in. It is available as a library for Rust, JavaScript/TypeScript, and Python.

It is built for syncing text files when all you have are snapshots: the document before editing and the two versions it became. The merge needs no saved operation history or persistent document metadata beyond that parent text.

## Try it

✨ **[Try the interactive demo][8]** to see it in action!

For the motivation and algorithm behind the library, read [Conflict-Free Three-Way Text Merging](https://schmelczer.dev/articles/reconcile-text-3-way-merge/).

### Install it in your project

- `cargo add reconcile-text` ([reconcile-text on crates.io][9])
- `npm install reconcile-text` ([reconcile-text on NPM][10])
- `uv add reconcile-text` or `pip install reconcile-text` ([reconcile-text on PyPI][27])

## Quick start

Each example merges two changes to the same greeting: one side adds a word, while the other changes the greeting itself.

### Rust

Install from [crates.io][crates]:

```sh
cargo add reconcile-text
```


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

See the [merge-file example](examples/merge-file.rs) for a file-merging CLI, or the [Rust API documentation][rust-docs].

### JavaScript/TypeScript

Install from [npm][npm]:

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

See the [TypeScript guide](docs/advanced-ts.md) for cursors, change provenance, and compact diffs, or the [demo website source](examples/website/src/index.ts) for an editor integration.

The Node.js and browser builds use the Rust library compiled to WebAssembly. For React Native, the package ships a pure-JavaScript build produced by [Binaryen's `wasm2js`](https://github.com/WebAssembly/binaryen) via its `react-native` entry point, so it can run in Hermes without a `WebAssembly` global.

### Python

Install from [PyPI][pypi] with uv or pip:

```sh
uv add reconcile-text
# or: pip install reconcile-text
```

Then use it in your application:

```python
from reconcile_text import reconcile

parent = "Hello world"
left = "Hello beautiful world"
right = "Hi world"

result = reconcile(parent, left, right)
print(result["text"])  # "Hi beautiful world"
```

The Python package provides native bindings to the same Rust implementation. See the [merge-file example](examples/merge_file.py) for a file-merging CLI, or the [Python guide](docs/advanced-python.md) for cursors, change provenance, and compact diffs.

## Motivation

Collaborative editing presents the challenge of merging conflicting changes when multiple users edit documents simultaneously or asynchronously whilst offline. Traditional solutions like Conflict-free Replicated Data Types (CRDTs) or Operational Transformation (OT) work well when you control the complete editing infrastructure and can capture every individual operation ([1]). However, many workflows involve users editing with various tools, for example, Obsidian users editing Markdown files with various editors ranging from Vim to VS Code.

This creates **Differential Synchronisation** scenarios ([2], [3]): we only know the final state of each document, not the sequence of operations that produced it. This is the same challenge Git addresses, but Git requires manual conflict resolution. The key insight is that while incorrect merges in source code can introduce bugs, human text is more forgiving: a slightly imperfect sentence is often preferable to conflict markers interrupting the flow.

> **Note**: Some text domains require more careful handling. Legal contracts, for instance, could have unintended meaning changes from conflicting edits that create double negations. At the same time, semantic conflicts can still arise when merging code, even in the absence of syntactic conflicts.

Differential sync is implemented by [universal-sync][12], and it requires a merging tool that creates conflict-free results for the best user experience.

## How it works

`reconcile-text` starts off similarly to `diff3` ([4], [5]) but adds automated conflict resolution. Given a **parent** document and two modified versions (`left` and `right`), the following happens:

1. **Tokenisation** - Input texts are split into meaningful units (words, characters, etc.) for granular merging
2. **Diff computation** - Myers' algorithm calculates differences between (parent ↔ left) and (parent ↔ right)
3. **Diff optimisation** - Operations are reordered and consolidated to maximise chained changes
4. **Operational Transformation** - Edits are woven together using OT principles, preserving all modifications and updating cursors

Whilst the primary goal of `reconcile-text` isn't to implement OT, it provides an elegant way to merge Myers' diff outputs. (For a dedicated Rust OT implementation, see [operational-transform-rs][13].) The same could be achieved with CRDTs, which many libraries implement well for text (see [Loro][14], [cola][15], and [automerge][16]).

However, when only the end result of concurrent changes is observable, merge quality depends entirely on the quality of the underlying 2-way diffs. For instance, `move` operations cannot be supported because Myers' algorithm decomposes them into separate `insert` and `delete` operations, regardless of the merging algorithm used.

## Comparison with other approaches

### Traditional 3-way merge (diff3, Git)

Tools like `diff3` ([4]) and Git produce **conflict markers** (`<<<<<<<` / `=======` / `>>>>>>>`) when both sides modify the same region. This works for source code where a human must verify correctness, but breaks the reading flow for prose. `reconcile-text` uses the same diff3-like foundation but adds an OT-inspired resolution step that eliminates conflict markers entirely. Libraries like [diffy][17], [merge3][18] (Rust), and [node-diff3][19] (JavaScript) all fall into this category.

### diff-match-patch

[diff-match-patch][6] is a widely-used library created by Neil Fraser at Google in 2006, providing character-level diffing (Myers' algorithm), fuzzy string matching (Bitap algorithm), and patch application. It powers Fraser's **Differential Synchronisation** protocol ([2]): compute a diff between two texts, apply the patch to a third text that may have drifted, and repeat until convergence. If a patch fails, the failure self-corrects in the next sync cycle.

The key differences from `reconcile-text`:

- **2-way vs 3-way** - diff-match-patch diffs two texts and applies the result as a patch. It has no concept of a common ancestor and cannot reason about "left changes" vs "right changes". `reconcile-text` performs true 3-way merging, understanding the intent behind each side's edits.

- **Character-level only** - Word-level and line-level diffs require encoding tokens as single Unicode characters before diffing ([7]). `reconcile-text` supports word, character, line, and custom tokenisation natively.

- **Patches can fail** - `patch_apply` returns a boolean array indicating success per patch. In Differential Synchronisation, failures self-correct in the next cycle, but for one-shot merges callers need to check the flags and handle failures. `reconcile-text` always produces a complete merged result.

- **No cursor tracking or change provenance** - diff-match-patch does not reposition cursors or track which side made which edit. `reconcile-text` does both automatically.

See the [comparison example](examples/compare-with-diff-match-patch.rs) for cases where diff-match-patch garbles overlapping edits and reports a failed sentence-insertion patch, alongside `reconcile-text`'s three-way merge results.

> **When to use diff-match-patch instead**: when you don't have a common ancestor, for example synchronising texts that have diverged through an unknown sequence of edits. If you have a common ancestor (as in most version control and collaborative editing scenarios), `reconcile-text` produces more reliable results.

### CRDTs (Yjs, Automerge, Loro, diamond-types)

Conflict-free Replicated Data Types ensure that replicas incorporating the same updates reach equivalent states according to the data type's defined conflict semantics, provided its delivery assumptions are met. In operation-based CRDTs, effectors (the updates applied at replicas) commute for concurrently invoked operations; the original operations need not commute, and causally dependent updates may require causal delivery. State-based CRDTs instead propagate and merge replica states ([28], sections 2.4 and 3.1).

Convergence does not guarantee that every edit remains visible or that every user's intent is preserved. For example, a last-writer-wins register selects a winner among competing writes ([28]).

Document CRDT libraries like [Yjs][20] (and its Rust port [Yrs][21]), [Automerge][16], [Loro][14], [cola][15], and [diamond-types][22] maintain replicated text structures with identities and causal metadata for merging concurrent changes. They are well suited to collaborative editing across multiple replicas when the application can integrate changes into those structures; this does not require recording every keystroke.

The trade-off for these document libraries is **maintaining document state over time**, including metadata beyond the visible text. For a workflow that only observes final text snapshots, using these structures entails translating differences into CRDT updates and managing CRDT state. `reconcile-text` instead accepts a parent and two edited strings directly.

> **When to use CRDTs instead**: if you can integrate a document CRDT and persist its state, these libraries provide convergence under their documented assumptions and naturally support multiple concurrent editors. `reconcile-text` merges exactly two forks at a time (though merges can be chained).

### Operational Transformation (OT)

OT libraries like [ot.js][23] and [ShareJS][24] transform concurrent operations against each other so that applying them in any order produces the same result. They work with individual operations and require infrastructure to coordinate them, typically a central server that determines the canonical operation order.

`reconcile-text` borrows the *concept* of OT (transforming one side's edits against the other) but applies it to a different problem. Instead of transforming individual keystrokes in real time, it transforms the consolidated diff output of two complete edits. This means it doesn't need a server, doesn't need to capture operations as they happen, and works entirely offline.

> **When to use OT instead**: if you need real-time collaboration with sub-second latency and can run a coordination server, dedicated OT libraries handle this well. `reconcile-text` is designed for merge points, not live keystroke-by-keystroke synchronisation.

## Development

Contributions are welcome!

### Environment

#### Python setup

Install [uv](https://docs.astral.sh/uv/getting-started/installation/) and build the extension for development:

```sh
cd reconcile-python
uv run maturin develop
```

#### Node.js setup

1. Install [nvm][nvm]:
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

Install [rustup][rustup]:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

### Scripts

- **Run tests**: `scripts/test.sh`
- **Check lint and formatting**: `scripts/lint.sh`
- **Develop demo website**: `scripts/dev-website.sh`
- **Build demo website**: `scripts/build-website.sh`
- **Publish new version**: `scripts/bump-version.sh patch`

## License

[MIT](./LICENSE)

[demo]: https://reconcile.schmelczer.dev
[article]: https://schmelczer.dev/articles/reconcile-text-3-way-merge/
[syncer]: https://schmelczer.dev/articles/reconcile-text-3-way-merge/#how-to-implement-a-syncer
[crates]: https://crates.io/crates/reconcile-text
[npm]: https://www.npmjs.com/package/reconcile-text
[pypi]: https://pypi.org/project/reconcile-text/
[rust-docs]: https://docs.rs/reconcile-text/latest/reconcile_text
[diff3]: https://blog.jcoglan.com/2017/05/08/merging-with-diff3/
[git-merge-drivers]: https://git-scm.com/docs/gitattributes#_built_in_merge_drivers
[diff-match-patch]: https://github.com/google/diff-match-patch
[yjs]: https://github.com/yjs/yjs
[automerge]: https://github.com/automerge/automerge
[loro]: https://github.com/loro-dev/loro/
[ot-js]: https://ot.js.org/
[nvm]: https://github.com/nvm-sh/nvm
[rustup]: https://rustup.rs
