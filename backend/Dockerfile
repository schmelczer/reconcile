FROM rust:1.83 as builder

WORKDIR /usr/src/backend

RUN apt update && apt install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl

COPY . .

RUN cargo build --package sync_server --release --target x86_64-unknown-linux-musl

FROM alpine:3.21.0

COPY --from=builder /usr/src/backend/target/x86_64-unknown-linux-musl/release/sync_server /app/sync_server

ENTRYPOINT ["/app/sync_server"]
