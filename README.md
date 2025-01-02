

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


## cut new version 

```sh
cd plugin
npm version patch
git tag -a 0.0.2 -m "0.0.2"
git push origin 0.0.2
```

npm install -g openapi-typescript
openapi-typescript http://localhost:3030/api.json --output plugin/src/services/types.ts


## Todos

- Add users to vaults
- Websocket for db updates
- async read body
- e2e tests
- add clap
- add auth middleware
- run eslint in ci

- CI for:
    - publish reconcile
    - cross-platform build server
    - run load test on server
    - build and publish plugin with openapi types
    - build docker image

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


update card title max width
reset should reset counters
access logs
retry
mem usage