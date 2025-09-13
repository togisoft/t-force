#!/bin/bash

# Script to restart the backend with watch mode
# This is useful when you want to restart just the backend without affecting other services

set -e

echo "🔄 Restarting backend with watch mode..."

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker and try again."
    exit 1
fi

# Check if docker-compose.dev.yml exists
if [ ! -f "docker-compose.dev.yml" ]; then
    echo "❌ docker-compose.dev.yml not found. Please run this script from the project root."
    exit 1
fi

echo "🛑 Stopping backend container..."
docker-compose -f docker-compose.dev.yml stop backend

echo "🧹 Removing backend container..."
docker-compose -f docker-compose.dev.yml rm -f backend

echo "🔧 Rebuilding and starting backend with watch mode..."
docker-compose -f docker-compose.dev.yml up --build -d backend

echo ""
echo "✅ Backend restarted with watch mode!"
echo "👀 Backend will automatically rebuild when you make changes to:"
echo "   - src/ directory"
echo "   - migration/ directory"
echo "   - Cargo.toml files"
echo ""
echo "📋 View backend logs:"
echo "   docker-compose -f docker-compose.dev.yml logs -f backend"
echo "" 