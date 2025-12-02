#!/bin/bash
# Build script for prediction market backend services
# Handles sqlx offline mode and ensures all services build correctly

set -e

echo "========================================="
echo "Building Prediction Market Backend"
echo "========================================="

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Check if .env file exists and load DATABASE_URL
if [ -f ".env" ]; then
    echo "✓ .env file found"
    # Load DATABASE_URL from .env file (handle comments and empty lines)
    while IFS= read -r line || [ -n "$line" ]; do
        # Skip comments and empty lines
        if [[ ! "$line" =~ ^[[:space:]]*# ]] && [[ -n "$line" ]]; then
            # Check if line contains DATABASE_URL
            if [[ "$line" == DATABASE_URL=* ]]; then
                export "$line"
                break
            fi
        fi
    done < .env
else
    echo "⚠️  Warning: .env file not found. Some services may fail to start."
    echo "   Make sure to create .env file with required environment variables."
fi

# Check if .sqlx directory exists for db_worker
SQLX_DIR="services/db_worker/.sqlx"
if [ ! -d "$SQLX_DIR" ]; then
    echo ""
    echo "❌ .sqlx directory not found for db_worker"
    echo ""
    
    if [ -z "$DATABASE_URL" ]; then
        echo "DATABASE_URL is not set. Cannot generate .sqlx metadata."
        echo ""
        echo "To fix this, you have two options:"
        echo ""
        echo "Option 1: Generate .sqlx metadata (if you have database access):"
        echo "  1. Set DATABASE_URL in .env file or export it:"
        echo "     export DATABASE_URL=\"postgresql://user:password@host:5432/database\""
        echo "  2. Run: cd services/db_worker && ./prepare-sqlx.sh"
        echo "  3. Then run this build script again: ./build.sh"
        echo ""
        echo "Option 2: Copy .sqlx directory from another machine:"
        echo "  1. On a machine with database access, run prepare-sqlx.sh"
        echo "  2. Copy the services/db_worker/.sqlx/ directory to this VM"
        echo "  3. Then run this build script again: ./build.sh"
        echo ""
        exit 1
    else
        echo "DATABASE_URL found. Generating .sqlx metadata..."
        echo ""
        cd services/db_worker
        
        # Make sure prepare-sqlx.sh is executable
        if [ ! -x "prepare-sqlx.sh" ]; then
            chmod +x prepare-sqlx.sh
        fi
        
        # Run prepare-sqlx.sh
        if ./prepare-sqlx.sh; then
            echo ""
            echo "✓ .sqlx metadata generated successfully"
            cd "$SCRIPT_DIR"
        else
            echo ""
            echo "❌ Failed to generate .sqlx metadata"
            echo "Please check your DATABASE_URL and database connectivity"
            cd "$SCRIPT_DIR"
            exit 1
        fi
    fi
else
    echo "✓ .sqlx directory found for db_worker"
fi

# Ensure SQLX_OFFLINE is set for the build
export SQLX_OFFLINE=true

echo ""
echo "Building all services in release mode..."
echo "This may take several minutes..."
echo ""

# Build all services
cargo build --release

echo ""
echo "========================================="
echo "✓ Build completed successfully!"
echo "========================================="
echo ""
echo "Built binaries:"
echo "  - target/release/db_worker"
echo "  - target/release/engine"
echo "  - target/release/prediction-market"
echo ""
echo "You can now start the services with:"
echo "  pm2 start ecosystem.config.js"
echo ""

