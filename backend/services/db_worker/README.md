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
   cd backend/services/db_worker
   export DATABASE_URL="postgresql://..."
   ./prepare-sqlx.sh
   cd ../../..
   cargo build
   ```

2. **Option 2: Use offline mode** (if database is not available):
   ```bash
   # CRITICAL: You MUST set SQLX_OFFLINE=true when building from workspace root
   export SQLX_OFFLINE=true
   cargo build
   ```
   
   Note: This requires the `.sqlx/` directory to exist in `backend/services/db_worker/`. If it doesn't exist, you'll need to generate it first on a machine with database access, then copy it to your VM/CI environment.

### Troubleshooting Build Errors

If you see errors like `prepared statement "sqlx_s_1" does not exist`:

1. **Ensure `SQLX_OFFLINE=true` is set** when building from the workspace root:
   ```bash
   export SQLX_OFFLINE=true
   cargo build
   ```

2. **Verify the `.sqlx/` directory exists** in `backend/services/db_worker/`:
   ```bash
   ls -la backend/services/db_worker/.sqlx/
   ```

3. **If the directory doesn't exist**, generate it first:
   ```bash
   cd backend/services/db_worker
   export DATABASE_URL="postgresql://..."
   ./prepare-sqlx.sh
   ```

4. **When building from workspace root**, always set `SQLX_OFFLINE=true`:
   ```bash
   export SQLX_OFFLINE=true
   cargo build --bin db_worker
   ```

**Note:** The `offline = true` in `Cargo.toml` is not always sufficient when building from a workspace root. Always set `SQLX_OFFLINE=true` environment variable for workspace builds.

## Configuration

The service is configured for offline mode in `Cargo.toml`:

```toml
[package.metadata.sqlx]
offline = true
```

This tells sqlx to use the `.sqlx/` directory for query validation instead of connecting to a database at compile time.

