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

2. **The `.sqlx/` directory will be generated** - This directory contains query metadata and should be kept local (it's already in `.gitignore`). You don't need to commit it.

### When Queries Change

If you modify any SQL queries in the codebase, you need to regenerate the metadata:

```bash
./prepare-sqlx.sh
```

### Building on VM/CI

**Important:** The `.sqlx/` directory is in `.gitignore` and should not be committed. For builds to work on VMs or CI/CD:

1. **Option 1: Generate `.sqlx/` during build** (if database is available):
   ```bash
   export DATABASE_URL="postgresql://..."
   ./prepare-sqlx.sh
   cargo build
   ```

2. **Option 2: Use offline mode** (if database is not available):
   ```bash
   export SQLX_OFFLINE=true
   cargo build
   ```
   
   Note: This requires the `.sqlx/` directory to exist. If it doesn't exist, you'll need to generate it first on a machine with database access, then copy it to your VM/CI environment.

**Note:** If you encounter errors, ensure the `.sqlx/` directory exists in the `db_worker` directory, or set `SQLX_OFFLINE=true` environment variable.

## Configuration

The service is configured for offline mode in `Cargo.toml`:

```toml
[package.metadata.sqlx]
offline = true
```

This tells sqlx to use the `.sqlx/` directory for query validation instead of connecting to a database at compile time.

