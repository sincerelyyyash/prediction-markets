#!/bin/bash
# Script to prepare sqlx offline query metadata
# This should be run once when the database schema is available
# It generates the .sqlx/ directory with query metadata for offline builds

set -e

echo "Preparing sqlx offline query metadata..."

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Navigate to workspace root (backend directory)
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo "Error: DATABASE_URL environment variable is not set"
    echo "Please set DATABASE_URL before running this script"
    exit 1
fi

# Convert pooler URL to direct connection URL if needed
# Connection poolers (Neon, Supabase, etc.) don't support prepared statements
DB_URL="$DATABASE_URL"
if [[ "$DB_URL" == *"-pooler"* ]]; then
    echo "⚠️  Detected pooler URL. Connection poolers don't support prepared statements."
    echo "Converting to direct connection URL (removing '-pooler' from hostname)..."
    DB_URL="${DB_URL//-pooler/}"
    echo ""
    echo "If this fails, you may need to:"
    echo "1. Get the direct connection URL from your database provider's dashboard"
    echo "2. Set SQLX_DATABASE_URL with the direct connection URL"
    echo ""
fi

# Allow override with SQLX_DATABASE_URL for direct connections
if [ -n "$SQLX_DATABASE_URL" ]; then
    echo "Using SQLX_DATABASE_URL for direct connection..."
    DB_URL="$SQLX_DATABASE_URL"
fi

# Check if sqlx-cli is installed
if ! command -v cargo-sqlx &> /dev/null; then
    echo "Installing sqlx-cli..."
    cargo install sqlx-cli --no-default-features --features postgres
fi

# Navigate to workspace root and run sqlx prepare for db_worker package
# This ensures all queries in db_worker are analyzed
cd "$WORKSPACE_ROOT"
echo "Running cargo sqlx prepare from workspace root..."
echo "Workspace: $WORKSPACE_ROOT"
echo "Targeting package: db_worker"
cargo sqlx prepare --database-url "$DB_URL" -- --package db_worker 2>&1

echo ""
echo "✓ sqlx offline metadata prepared successfully!"
echo ""
echo "The .sqlx/ directory has been generated in services/db_worker/. Builds will now work offline without requiring a database connection."
