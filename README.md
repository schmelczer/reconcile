## VaultLink self-hosted Obsidian sync plugin

[![Check](https://github.com/schmelczer/vault-link/actions/workflows/check.yml/badge.svg)](https://github.com/schmelczer/vault-link/actions/workflows/check.yml)
[![Publish server Docker image](https://github.com/schmelczer/vault-link/actions/workflows/publish-docker.yml/badge.svg)](https://github.com/schmelczer/vault-link/actions/workflows/publish-docker.yml)
[![Publish Obsidian plugin](https://github.com/schmelczer/vault-link/actions/workflows/publish-plugin.yml/badge.svg)](https://github.com/schmelczer/vault-link/actions/workflows/publish-plugin.yml)


## Install [nvm](https://github.com/nvm-sh/nvm)

- `curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash`
- `nvm install 20`
- `nvm use 20`
- Optionally set the system-wide default: `nvm alias default 20`


## Set up Rust

- Install [`rustup`](https://rustup.rs): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- `sudo apt install llvm -y`
- `rustup self update`
- `rustup update`
- `rustup install nightly`
- `rustup default nightly`
- `rustup component add llvm-tools-preview`
- `cargo install cargo-generate cargo-fuzz cargo-insta rustfilt cargo-binutils`
- Install [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer): `curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`
- `cargo install cargo-insta sqlx-cli cargo-edit`


## Publish new version 

```sh
./bump-version.sh patch
```


## Update HTTP API TS bindings

```sh 
npm install -g openapi-typescript
openapi-typescript http://localhost:3030/api.json --output plugin/src/services/types.ts
```

```
## Todos

- Add users to vaults
- Websocket for db updates
- async read body
- e2e tests
- add clap
- add auth middleware
- shard db per user
- update card title max width
- retry
- CI for:
    - publish reconcile
    - cross-platform build server
    - run load test on server
    - build and publish plugin with openapi types

todo: enable
[workspace.lints.clippy]
single_call_fn = { level = "allow", priority = 1 }
absolute_paths = { level = "allow", priority = 1 }
arithmetic_side_effects = { level = "allow", priority = 1 }
similar_names = { level = "allow", priority = 1 }
self_named_module_files = { level = "allow", priority = 1 }
single_char_lifetime_names = { level = "allow", priority = 1 }
missing_docs_in_private_items = { level = "allow", priority = 1 }
question_mark_used =  { level = "allow", priority = 1 }
implicit_return = { level = "allow", priority = 1 }
pedantic = { level = "warn", priority = 0 }
cargo = { level = "warn", priority = 0 }

```

apt install flatpak
flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak install flathub md.obsidian.Obsidian
flatpak run md.obsidian.Obsidian


stop leaking subscriptions
test with naughty strings https://github.com/minimaxir/big-list-of-naughty-strings/tree/84a5dea833b5e2218f7c8c2104effca3f8f155aa?tab=readme-ov-file
double check internalSyncRemotelyUpdatedFile