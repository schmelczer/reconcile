# `reconcile-text`: conflict-free 3-way text merging

Merge a common parent and two edited versions into one text, without conflict markers. `reconcile-text` combines changes from both sides and updates any cursor positions you pass in. It is available as a library for Rust, JavaScript/TypeScript, and Python.

It is built for syncing text files when all you have are snapshots: the document before editing and the two versions it became. The merge needs no saved operation history or persistent document metadata beyond that parent text.

## Try it

✨ **[Try the interactive demo][demo]** to see it in action!

For the motivation and algorithm behind the library, read [Conflict-Free Three-Way Text Merging][article].

### Install it in your project

- `cargo add reconcile-text` ([reconcile-text on crates.io][crates])
- `npm install reconcile-text` ([reconcile-text on NPM][npm])
- `uv add reconcile-text` or `pip install reconcile-text` ([reconcile-text on PyPI][pypi])

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

See the [merge-file example][rust-merge-file] for a file-merging CLI, or the [Rust API documentation][rust-docs].

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

See the [TypeScript guide][typescript-guide] for cursors, change provenance, and compact diffs, or the [demo website source][demo-source] for an editor integration.

The Node.js and browser builds use the Rust library compiled to WebAssembly. For React Native, the package ships a pure-JavaScript build produced by [Binaryen's `wasm2js`][binaryen] via its `react-native` entry point, so it can run in Hermes without a `WebAssembly` global.

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

The Python package provides native bindings to the same Rust implementation. See the [merge-file example][python-merge-file] for a file-merging CLI, or the [Python guide][python-guide] for cursors, change provenance, and compact diffs.

## Why merge snapshots?

Collaborative editing presents the challenge of merging conflicting changes when multiple users edit documents simultaneously or asynchronously whilst offline. A note might be edited in Obsidian on one device and Vim on another, with a file sync service carrying changes between them. By the time those changes meet, the syncer sees two files. It cannot ask either editor for the operations that produced them.

A common ancestor gives it enough context to compare both sets of edits. Git works with the same inputs, but usually leaves ambiguous conflicts for a person to resolve. That is a useful default for source code. For notes and prose, an automatic merge can be more useful, even if the wording occasionally needs tidying up.

## How it works

`reconcile-text` starts off similarly to [`diff3`][diff3] but adds automated conflict resolution. Given a **parent** document and two modified versions (`left` and `right`), the following happens:

1. **Tokenisation** - Input texts are split into meaningful units (words, characters, etc.) for granular merging
2. **Diff computation** - Myers' algorithm calculates differences between (parent ↔ left) and (parent ↔ right)
3. **Diff optimisation** - Operations are reordered and consolidated to maximise chained changes
4. **Operational Transformation** - Edits are woven together using OT principles, preserving all modifications and updating cursors

Whilst the primary goal of `reconcile-text` isn't to implement OT, it provides an elegant way to merge Myers' diff outputs. (For a dedicated Rust OT implementation, see [operational-transform-rs][operational-transform-rs].) The same could be achieved with CRDTs, which many libraries implement well for text (see [Loro][loro], [cola][cola], and [automerge][automerge]).

However, when only the end result of concurrent changes is observable, merge quality depends entirely on the quality of the underlying 2-way diffs. For instance, `move` operations cannot be supported because Myers' algorithm decomposes them into separate `insert` and `delete` operations, regardless of the merging algorithm used.

### Choosing a tokeniser

Tokenisation affects the result, especially when edits overlap. The built-in options are:

- **Word**: keeps words together; the default in JavaScript/TypeScript and Python.
- **Character**: merges individual Unicode code points for finer-grained changes.
- **Line**: treats each line as a unit.
- **Markdown**: builds on word tokenisation, preserving newlines and attaching block prefixes such as list markers to their first word.

Rust callers choose a tokeniser explicitly and can also supply a custom function. The merge keeps tokens intact, but the Markdown tokeniser does not validate the resulting document's syntax. Try different tokenisers in the [demo][demo] to see how they handle your text.

## Comparison with other approaches

### diff3 and Git

`diff3` and Git's default text merge leave conflict markers where changes cannot be combined automatically. Git also has a [`union` merge driver][git-merge-drivers] that retains lines from both sides of a conflict. Because it works with whole lines, two edits to different words in a sentence can leave two versions of the sentence. Word tokenisation lets `reconcile-text` combine those edits within the line.

### diff-match-patch

[diff-match-patch][diff-match-patch] computes two-way diffs and applies patches using fuzzy matching. It is useful when you need to apply changes to text whose context may have shifted. A patch can fail, so callers need to inspect its success flags.

`reconcile-text` uses the common parent to derive both sides' changes before combining them. The [comparison example][comparison-example] shows how the two approaches handle overlapping replacements and a sentence inserted into a rewritten paragraph.

### CRDTs and OT

Libraries based on conflict-free replicated data types (CRDTs), such as [Yjs][yjs], [Automerge][automerge], and [Loro][loro], maintain document structures and metadata that let replicas merge updates. They fit applications that can persist that state and integrate edits into it. Snapshot changes can also be translated into CRDT updates; using a CRDT does not require recording every keystroke.

OT libraries such as [ot.js][ot-js] work with operations and a protocol for coordinating them. `reconcile-text` uses OT ideas internally, deriving the operations from each pair of snapshots at merge time.

The practical distinction is what your application keeps between merges. `reconcile-text` accepts three texts and merges two forks at a time. A collaborative editing system built around a CRDT or OT also manages the state and protocol needed to coordinate replicas over time.

## Development

Contributions are welcome!

### Environment

#### Python setup

Install [uv][uv] and build the extension for development:

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

[MIT][license]

[demo]: https://reconcile.schmelczer.dev/
[article]: https://schmelczer.dev/articles/reconcile-text-3-way-merge/
[crates]: https://crates.io/crates/reconcile-text
[npm]: https://www.npmjs.com/package/reconcile-text
[pypi]: https://pypi.org/project/reconcile-text/
[rust-merge-file]: examples/merge-file.rs
[rust-docs]: https://docs.rs/reconcile-text/latest/reconcile_text/
[typescript-guide]: docs/advanced-ts.md
[demo-source]: examples/website/src/index.ts
[binaryen]: https://github.com/WebAssembly/binaryen
[python-merge-file]: examples/merge_file.py
[python-guide]: docs/advanced-python.md
[diff3]: https://blog.jcoglan.com/2017/05/08/merging-with-diff3/
[operational-transform-rs]: https://github.com/spebern/operational-transform-rs
[loro]: https://github.com/loro-dev/loro
[cola]: https://github.com/noib3/cola
[automerge]: https://github.com/automerge/automerge
[git-merge-drivers]: https://git-scm.com/docs/gitattributes#_built_in_merge_drivers
[diff-match-patch]: https://github.com/google/diff-match-patch
[comparison-example]: examples/compare-with-diff-match-patch.rs
[yjs]: https://github.com/yjs/yjs
[ot-js]: https://github.com/Operational-Transformation/ot.js
[uv]: https://docs.astral.sh/uv/getting-started/installation/
[nvm]: https://github.com/nvm-sh/nvm
[rustup]: https://rustup.rs
[license]: ./LICENSE
