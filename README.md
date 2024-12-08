

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
- `cargo install cargo-insta`
- `cargo install sqlx-cli`





## Todos

- Add users to vaults
- Websocket for db updates
- async read body
- e2e tests
- add clap
- add auth middleware
- add request logs
- CI for:
    - publish reconcile
    - cross-platform build server
    - run load test on server
    - build and publish plugin with openapi types
    - build docker image
