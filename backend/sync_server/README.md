
cargo install sqlx-cli
rm db.sqlite3; sqlx database create --database-url sqlite://db.sqlite3
sqlx migrate run --source sync_server/src/database/migrations --database-url sqlite://db.sqlite3
