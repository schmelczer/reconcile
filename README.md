# VaultLink self-hosted Obsidian plugin for file syncing

[![Check](https://github.com/schmelczer/vault-link/actions/workflows/check.yml/badge.svg)](https://github.com/schmelczer/vault-link/actions/workflows/check.yml)
[![E2E tests](https://github.com/schmelczer/vault-link/actions/workflows/e2e.yml/badge.svg)](https://github.com/schmelczer/vault-link/actions/workflows/e2e.yml)
[![Publish server Docker image](https://github.com/schmelczer/vault-link/actions/workflows/publish-docker.yml/badge.svg)](https://github.com/schmelczer/vault-link/actions/workflows/publish-docker.yml)
[![Publish Obsidian plugin](https://github.com/schmelczer/vault-link/actions/workflows/publish-plugin.yml/badge.svg)](https://github.com/schmelczer/vault-link/actions/workflows/publish-plugin.yml)


## Develop

### Install [nvm](https://github.com/nvm-sh/nvm)

- `curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash`
- `nvm install 22`
- `nvm use 22`
- Optionally set the system-wide default: `nvm alias default 22`


### Set up Rust

- Install [`rustup`](https://rustup.rs): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Install [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer): `curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`
- `cargo install cargo-insta sqlx-cli cargo-edit`

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

- [Sync server](./backend/sync_server/README.md)
