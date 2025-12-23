Antes de ejecutar sqlx migrate run, se debe tener el archivo de migración:
migrations/20251223_create_users.sql

```
sqlx migrate run
```

```
cargo run --bin seed_dev
```
```
cargo run --bin auth-api-rust
```