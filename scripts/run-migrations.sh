#!/bin/bash

# Script to run database migrations
# This script uses the migration binary instead of sea-orm-cli

set -e

echo "🗄️  Running database migrations..."

# Check if we're in the right directory
if [ ! -f "docker-compose.dev.yml" ]; then
    echo "❌ docker-compose.dev.yml not found. Please run this script from the project root."
    exit 1
fi

# Check if the database is running
if ! docker-compose -f docker-compose.dev.yml ps db | grep -q "Up"; then
    echo "❌ Database is not running. Starting it first..."
    docker-compose -f docker-compose.dev.yml up -d db
    
    # Wait for database to be ready
    echo "⏳ Waiting for database to be ready..."
    sleep 10
fi

# Build and run migrations using the migration binary
echo "🔧 Building and running migrations..."

# Option 1: Run migrations using the migration binary
if [ -f "backend/migration/Cargo.toml" ]; then
    echo "📦 Using migration binary..."
    cd backend/migration
    cargo run -- up
    cd ../..
else
    echo "❌ Migration binary not found. Please ensure migration/Cargo.toml exists."
    exit 1
fi

echo "✅ Migrations completed successfully!"
echo ""
echo "📋 Database status:"
echo "   Host: localhost"
echo "   Port: 5433"
echo "   Database: ${POSTGRES_DB:-tforce}"
echo "   User: ${POSTGRES_USER:-postgres}"
echo "" 