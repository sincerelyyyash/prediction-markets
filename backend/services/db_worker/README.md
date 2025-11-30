# DB Worker Service

Database worker service that handles database operations for the prediction market system.

## SQLx Offline Mode Setup

This service uses `sqlx` with compile-time query checking. For builds to work on VMs or CI/CD environments where the database might not be available, the service is configured for offline mode.

### Initial Setup

1. **Generate SQLx metadata** (requires database access):
   ```bash
   # Set your database connection URL (use direct connection, not pooler)
   # ⚠️  Never commit credentials to the repository!
   export DATABASE_URL="postgresql://user:password@host:5432/database_name"
   ./prepare-sqlx.sh
   ```

   **Security Note:** Never commit database credentials or connection strings to the repository. Always use environment variables or `.env` files (which should be in `.gitignore`).

   **Important for Connection Poolers (Neon, Supabase, etc.):**
   - Connection poolers don't support prepared statements, which are required for `cargo sqlx prepare`
   - You **must** use the **direct connection URL** (not the pooler URL)
   - The script will attempt to auto-convert pooler URLs by removing `-pooler` from the hostname
   - If auto-conversion fails:
     1. Get your direct connection URL from your database provider's dashboard
     2. Use it directly: `export DATABASE_URL="postgresql://..."` (without `-pooler` in hostname)
     3. Or set `SQLX_DATABASE_URL` with the direct URL: `export SQLX_DATABASE_URL="postgresql://..."`

2. **Commit the generated `.sqlx/` directory**:
   ```bash
   git add .sqlx/
   git commit -m "Add sqlx offline query metadata"
   ```

### When Queries Change

If you modify any SQL queries in the codebase, you need to regenerate the metadata:

```bash
./prepare-sqlx.sh
git add .sqlx/
git commit -m "Update sqlx offline query metadata"
```

### Building on VM/CI

Once the `.sqlx/` directory is committed, builds will work offline without requiring a database connection:

```bash
cargo build
```

**Note:** If you still encounter errors, ensure the `.sqlx/` directory exists in the `db_worker` directory. You can also set the `SQLX_OFFLINE=true` environment variable as an additional safeguard, though it should not be necessary if the `.sqlx/` directory is present.

## Configuration

The service is configured for offline mode in `Cargo.toml`:

```toml
[package.metadata.sqlx]
offline = true
```

This tells sqlx to use the `.sqlx/` directory for query validation instead of connecting to a database at compile time.

