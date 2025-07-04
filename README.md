# Reconcile: conflict-free 3-way text merging

> `diff3` but with automatic conflict resolution.

## Features

-   Conflict-free output (no more git conflict markers like in )
-   Support for updating cursor/selection positions
-   Pluggable tokenizer
-   Full UTF-8 support
-   WASM

## Motivation

Sometimes documents get edited concurrently by multiple users (or the same user from multiple devices) resulting in divergent changes.

To allow for offline editing, we could use CRDTs or Operational Transformation (OT) to come to a consistent resolution of the competing version. However, this requires capturing all user actions: insertions, deletes, move, copies, and pastes. In some application, this is trivial if the document can only be edited through an editor somehow in our control. But this isn't always the case. Users enjoy composable systems that don't lock them in. For example, one of the unique selling points of Obsidian is to provide an editor experience over a folder Markdown files leaving the user free to change their technology of choice on a whim.

This means that files can be edited out-of-channel and the only information a text synchronisation system can know is the current content of each tracked file. This is the same problem as what Git and similar version control systems solve. Although the problem is similar, there's a relevant difference between syncing source code and personal notes: in the case of the former, a semantically incorrect conflict resolution can wreak havoc in a code base, or worse, introduce a correctness bug unnoticed. Text notes are different though, humans are well-equipped to finding the signal in a noisy environment and "bad merges" might result in a clumsy sentence but the reader will likely still understand the gist and can fix it if necessary.

> There are domains of human text which are less tolerant of mis-merges: for instance, a two conflicting changes to a contract could result in a term getting negated in different ways from both sides, resulting in a double-negation, thus, unknowingly changing the meaning.

# VaultLink self-hosted Obsidian plugin for file syncing

[![Check](https://github.com/schmelczer/reconcile/actions/workflows/check.yml/badge.svg)](https://github.com/schmelczer/reconcile/actions/workflows/check.yml)
[![Publish to GitHub Pages](https://github.com/schmelczer/reconcile/actions/workflows/gh-pages.yml/badge.svg)](https://github.com/schmelczer/reconcile/actions/workflows/gh-pages.yml)

## Develop

### Install [nvm](https://github.com/nvm-sh/nvm)

-   `curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash`
-   `nvm install 22`
-   `nvm use 22`
-   Optionally set the system-wide default: `nvm alias default 22`

### Set up Rust

-   Install [`rustup`](https://rustup.rs): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
-   Install [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer): `curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`
-   `cargo install cargo-insta sqlx-cli cargo-edit`

### Install Obsidian on Linux

```sh
apt install flatpak
flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak install flathub md.obsidian.Obsidian
flatpak run md.obsidian.Obsidian
```

### Scripts

#### Update HTTP API TS bindings

```sh
scripts/update-api-types.sh
```

#### Publish new version

```sh
scripts/bump-version.sh patch
```

#### Run E2E tests

```sh
scripts/e2e.sh
```

And to clean up the logs & database files, run `scripts/clean-up.sh`

## Projects

-   [Sync server](./backend/sync_server/README.md)

npm install -g typescript
