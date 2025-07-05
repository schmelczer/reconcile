# Reconcile: conflict-free 3-way text merging

> `diff3` but with automatic conflict resolution.

[![Check](https://github.com/schmelczer/reconcile/actions/workflows/check.yml/badge.svg)](https://github.com/schmelczer/reconcile/actions/workflows/check.yml)
[![Publish to GitHub Pages](https://github.com/schmelczer/reconcile/actions/workflows/gh-pages.yml/badge.svg)](https://github.com/schmelczer/reconcile/actions/workflows/gh-pages.yml)

Reconcile is a Rust and JavaScript (through WebAssembly) library for merging text without user intervention. It automatically resolves conflicts that would typically require manual intervention in traditional 3-way merge tools.

```rust
use reconcile::{reconcile, BuiltinTokenizer};

let parent = "Merging text is hard!";
let left = "Merging text is easy!";
let right = "With reconcile, merging documents is hard!";

let deconflicted = reconcile(parent, &left.into(), &right.into(), &*BuiltinTokenizer::Word);
assert_eq!(deconflicted.apply().text(), "With reconcile, merging documents is easy!");
```

## Features

-   **Conflict-free output** - No more git conflict markers in the result
-   **Cursor/selection position tracking** - Automatically updates cursor positions during merging
-   **Pluggable tokenizer** - Choose between word-level, character-level, or custom tokenization
-   **Full UTF-8 support** - Handles Unicode text correctly
-   **WebAssembly support** - Use from JavaScript/TypeScript applications

## Quick Start

### Rust

Add to your `Cargo.toml`:
```toml
[dependencies]
reconcile = "0.4"
```

```rust
use reconcile::{reconcile, BuiltinTokenizer};

let parent = "Hello world";
let left = "Hello beautiful world";
let right = "Hi world";

let result = reconcile(parent, &left.into(), &right.into(), &*BuiltinTokenizer::Word);
assert_eq!(result.apply().text(), "Hi beautiful world");
```

### JavaScript/TypeScript

```bash
npm install reconcile
```

```javascript
import { init, reconcile } from 'reconcile';

// Initialize the WASM module (required before first use)
await init();

const parent = "Hello world";
const left = "Hello beautiful world";
const right = "Hi world";

const result = reconcile(parent, left, right);
console.log(result.text); // "Hi beautiful world"
```

## API

### Tokenizers

Reconcile supports different tokenization strategies:

- **Word tokenizer** (`BuiltinTokenizer::Word`): Splits text into words (default, recommended for most use cases)
- **Character tokenizer** (`BuiltinTokenizer::Character`): Splits text into individual characters (fine-grained merging)
- **Custom tokenizer**: Implement your own tokenization logic

### Cursor Tracking

Reconcile can automatically update cursor and selection positions during merging:

```javascript
const result = reconcile(
    "Hello world",
    {
        text: "Hello beautiful world",
        cursors: [{ id: 1, position: 6 }]  // After "Hello "
    },
    {
        text: "Hi world",
        cursors: [{ id: 2, position: 0 }]   // At beginning
    }
);

// Result includes updated cursor positions
console.log(result.cursors); // [{ id: 1, position: 3 }, { id: 2, position: 0 }]
```

### History Tracking

Use `reconcileWithHistory` to get detailed information about the merge process:

```javascript
const result = reconcileWithHistory(parent, left, right);
console.log(result.history); // Array of spans with their origins
```

## Algorithm

The algorithm starts similarly to `diff3`. Its inputs are a **parent** document and two conflicting versions: `left` and `right` which have been created from the parent through any series of concurrent edits.

1. **Diff calculation**: First, 2-way diffs of (parent & left) and (parent & right) are computed using Myers' algorithm
2. **Tokenization**: The text is split into tokens (words, characters, etc.) for granular merging
3. **Operation transformation**: The resulting edits are weaved together using operational transformation principles, ensuring no changes are lost
4. **Conflict resolution**: Unlike traditional 3-way merge tools, Reconcile automatically resolves conflicts without producing conflict markers

The key insight is that both insertions and deletions are preserved: if either side inserted text, it appears in the result; if either side deleted text, the deletion is applied, but insertions into deleted regions are still preserved.

## Motivation

Sometimes documents get edited concurrently by multiple users (or the same user from multiple devices) resulting in divergent changes.

To allow for offline editing, we could use CRDTs or Operational Transformation (OT) to come to a consistent resolution of the competing version. However, this requires capturing all user actions: insertions, deletes, move, copies, and pastes. In some applications, this is trivial if the document can only be edited through an editor that's in our control. But this isn't always the case. Users enjoy composable systems that don't lock them in. For example, one of the unique selling points of Obsidian is to provide an editor experience over a folder of Markdown files leaving the user free to change their technology of choice on a whim.

This means that files can be edited out-of-channel and the only information a text synchronization system can know is the current content of each tracked file. This is the same problem as what Git and similar version control systems solve. Although the problem is similar, there's a relevant difference between syncing source code and personal notes: in the case of the former, a semantically incorrect conflict resolution can wreak havoc in a code base, or worse, introduce a correctness bug unnoticed. Text notes are different though, humans are well-equipped to finding the signal in a noisy environment and "bad merges" might result in a clumsy sentence but the reader will likely still understand the gist and can fix it if necessary.

> There are domains of human text which are less tolerant of mis-merges: for instance, two conflicting changes to a contract could result in a term getting negated in different ways from both sides, resulting in a double-negation, thus unknowingly changing the meaning.

## Development

### Prerequisites

#### Install Node.js
-   Install [nvm](https://github.com/nvm-sh/nvm): `curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash`
-   `nvm install 22`
-   `nvm use 22`
-   Optionally set the system-wide default: `nvm alias default 22`

#### Set up Rust
-   Install [`rustup`](https://rustup.rs): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
-   Install [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer): `curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`
-   `cargo install cargo-insta cargo-edit`

### Building

```bash
# Build Rust library
cargo build

# Build WASM bindings
wasm-pack build --target web

# Build JavaScript package
cd reconcile-js
npm install
npm run build
```

### Testing

```bash
# Test Rust library
cargo test

# Test JavaScript bindings
cd reconcile-js
npm test
```

### Scripts

#### Publish new version
```sh
scripts/bump-version.sh patch
```

## License

MIT
