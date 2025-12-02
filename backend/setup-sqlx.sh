#!/bin/bash
# Quick script to generate .sqlx metadata for db_worker
# Run this before building if .sqlx directory doesn't exist

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "========================================="
echo "Setting up SQLx offline metadata"
echo "========================================="
echo ""

# Check if .env file exists and load DATABASE_URL
if [ -f ".env" ]; then
    echo "Loading DATABASE_URL from .env file..."
    while IFS= read -r line || [ -n "$line" ]; do
        if [[ ! "$line" =~ ^[[:space:]]*# ]] && [[ -n "$line" ]]; then
            if [[ "$line" == DATABASE_URL=* ]]; then
                export "$line"
                break
            fi
        fi
    done < .env
fi

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo "❌ Error: DATABASE_URL is not set"
    echo ""
    echo "Please set DATABASE_URL in one of these ways:"
    echo ""
    echo "1. Add to .env file:"
    echo "   DATABASE_URL=postgresql://user:password@host:5432/database"
    echo ""
    echo "2. Export as environment variable:"
    echo "   export DATABASE_URL=\"postgresql://user:password@host:5432/database\""
    echo ""
    echo "Note: Use the DIRECT connection URL (not pooler URL) for sqlx prepare"
    echo ""
    exit 1
fi

echo "✓ DATABASE_URL is set"
echo ""

# Check if .sqlx already exists
SQLX_DIR="services/db_worker/.sqlx"
if [ -d "$SQLX_DIR" ]; then
    echo "⚠️  .sqlx directory already exists"
    read -p "Regenerate it? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Skipping..."
        exit 0
    fi
    echo ""
fi

# Navigate to db_worker directory
cd services/db_worker

# Make sure prepare-sqlx.sh is executable
if [ ! -x "prepare-sqlx.sh" ]; then
    chmod +x prepare-sqlx.sh
fi

# Run prepare-sqlx.sh
echo "Generating .sqlx metadata..."
echo ""
./prepare-sqlx.sh

echo ""
echo "========================================="
echo "✓ SQLx metadata setup complete!"
echo "========================================="
echo ""
echo "You can now build the project with:"
echo "  ./build.sh"
echo ""

