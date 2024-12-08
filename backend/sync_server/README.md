
cargo install sqlx-cli
rm sync_server/test.db && sqlx database create --database-url sqlite://sync_server/test.db
sqlx migrate run --source sync_server/src/database/migrations --database-url sqlite://sync_server/test.db
DATABASE_URL=sqlite://sync_server/test.db cargo sqlx prepare --workspace