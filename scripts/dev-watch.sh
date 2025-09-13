#!/bin/bash

# Development script with watch mode for backend
# This script starts the Docker development environment with real-time backend rebuilding

set -e

echo "🚀 Starting T-Force development environment with watch mode..."
echo "📝 Backend will automatically rebuild when files change"
echo ""

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

echo "🔧 Building and starting development containers..."
echo ""

# Build and start the development environment
docker compose -f docker-compose.dev.yml up --build -d

echo ""
echo "✅ Development environment started!"
echo ""
echo "📊 Services:"
echo "   🌐 Frontend: http://localhost:3000"
echo "   🔧 Backend API: http://localhost:8081"
echo "   🗄️  Database: localhost:5433"
echo "   📈 Traefik Dashboard: http://localhost:8080"
echo ""
echo "👀 Backend is running in watch mode - it will automatically rebuild when you make changes!"
echo ""
echo "📋 Useful commands:"
echo "   View logs: ./scripts/logs.sh"
echo "   Stop services: docker compose -f docker-compose.dev.yml down"
echo "   Restart backend: docker compose -f docker-compose.dev.yml restart backend"
echo ""

# Show logs for a few seconds to see the startup
echo "📋 Showing startup logs (press Ctrl+C to stop viewing logs):"
echo ""
docker compose -f docker-compose.dev.yml logs -f --tail=50