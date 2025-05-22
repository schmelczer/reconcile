# Sync server

## Creating/resetting the Database for development

```sh
sqlx database create --database-url sqlite://db.sqlite3
sqlx migrate run --source sync_server/src/app_state/database/migrations --database-url sqlite://db.sqlite3
cargo sqlx prepare --workspace
```
