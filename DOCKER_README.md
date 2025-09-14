# T-Force Docker Scripts Documentation

## 📋 Overview

This directory contains all the essential scripts for managing the T-Force application in both development and production environments. All scripts are designed to be modern, clean, and user-friendly.

## 🚀 Quick Start

### Development Environment
```bash
# Start development environment
./scripts/dev.sh

# Watch for file changes and auto-restart
./scripts/dev-watch.sh
```

### Production Deployment
```bash
# Deploy to production
./scripts/prod-deploy.sh
```

## 📁 Available Scripts

### 🔧 Development Scripts

#### `dev.sh`
**Purpose:** Start the development environment with all necessary services.

**Features:**
- ✅ Prerequisites checking
- ✅ Environment file validation
- ✅ Clean startup process
- ✅ Service health monitoring
- ✅ Development summary

**Usage:**
```bash
./scripts/dev.sh
```

**What it does:**
1. Checks if Docker is running
2. Validates `.env.dev` file
3. Stops existing development containers
4. Starts all development services
5. Waits for services to be ready
6. Shows service URLs and useful commands

---

#### `dev-watch.sh`
**Purpose:** Monitor file changes and automatically restart services.

**Features:**
- ✅ File watching with `fswatch`
- ✅ Backend-only watching
- ✅ Frontend-only watching
- ✅ All services watching
- ✅ Fallback to manual mode

**Usage:**
```bash
# Watch all changes
./scripts/dev-watch.sh

# Watch only backend changes
./scripts/dev-watch.sh backend

# Watch only frontend changes
./scripts/dev-watch.sh frontend
```

**Requirements:**
- `fswatch` (install with: `brew install fswatch`)

---

### 🏭 Production Scripts

#### `prod-deploy.sh`
**Purpose:** Professional production deployment with health checks and monitoring.

**Features:**
- ✅ Environment validation
- ✅ Service cleanup
- ✅ Health checks
- ✅ Database migrations
- ✅ Service monitoring
- ✅ Deployment summary

**Usage:**
```bash
./scripts/prod-deploy.sh
```

**What it does:**
1. Validates production environment
2. Stops existing production containers
3. Builds and starts production services
4. Runs database migrations
5. Performs health checks
6. Shows deployment summary

---

### 📊 Utility Scripts

#### `logs.sh`
**Purpose:** View logs for all services with smart environment detection.

**Features:**
- ✅ Auto environment detection (dev/prod)
- ✅ Service-specific logging
- ✅ Real-time log streaming
- ✅ Clean error handling

**Usage:**
```bash
# View all logs
./scripts/logs.sh

# View specific service logs
./scripts/logs.sh backend
./scripts/logs.sh frontend
./scripts/logs.sh db
./scripts/logs.sh traefik
./scripts/logs.sh grafana
./scripts/logs.sh prometheus
```

---

#### `backup.sh`
**Purpose:** Create database backups with timestamping.

**Features:**
- ✅ Timestamped backups
- ✅ Multiple backup formats
- ✅ Backup validation
- ✅ Cleanup of old backups

**Usage:**
```bash
./scripts/backup.sh
```

---

#### `stop.sh`
**Purpose:** Stop all running services.

**Features:**
- ✅ Graceful shutdown
- ✅ Service cleanup
- ✅ Environment detection

**Usage:**
```bash
./scripts/stop.sh
```

---

#### `reset.sh`
**Purpose:** Reset the entire environment (use with caution).

**Features:**
- ✅ Complete cleanup
- ✅ Volume removal
- ✅ Fresh start

**Usage:**
```bash
./scripts/reset.sh
```

## 🌐 Service URLs

### Development Environment
- **Frontend:** http://localhost:3000
- **Backend:** http://localhost:8080
- **Database:** localhost:5432

### Production Environment
- **Frontend:** http://localhost (via Traefik)
- **Backend:** http://localhost/api (via Traefik)
- **Grafana:** http://grafana.localhost
- **Database:** localhost:5432

## 🔧 Environment Files

### Development
- **`.env.dev`** - Development environment variables
- **`docker-compose.dev.yml`** - Development services

### Production
- **`.env.prod`** - Production environment variables
- **`docker-compose.prod.yml`** - Production services

## 📋 Prerequisites

### Required Software
- **Docker** - Container runtime
- **Docker Compose** - Service orchestration
- **fswatch** - File watching (for dev-watch.sh)

### Installation
```bash
# Install fswatch (macOS)
brew install fswatch

# Install fswatch (Ubuntu/Debian)
sudo apt-get install fswatch

# Install fswatch (CentOS/RHEL)
sudo yum install fswatch
```

## 🚨 Troubleshooting

### Common Issues

#### 1. Docker Not Running
```bash
# Start Docker Desktop
open -a Docker

# Or start Docker service (Linux)
sudo systemctl start docker
```

#### 2. Environment File Missing
```bash
# Create .env.dev from .env.example
cp .env.example .env.dev

# Create .env.prod from .env.example
cp .env.example .env.prod
```

#### 3. Port Conflicts
```bash
# Check what's using the port
lsof -i :3000
lsof -i :8080
lsof -i :5432

# Stop conflicting services
./scripts/stop.sh
```

#### 4. Permission Issues
```bash
# Make scripts executable
chmod +x scripts/*.sh
```

### Getting Help

#### Script Help
```bash
# Show script usage
./scripts/dev-watch.sh help
./scripts/logs.sh help
```

#### Service Status
```bash
# Check running services
docker compose -f docker-compose.dev.yml ps
docker compose -f docker-compose.prod.yml ps
```

#### View Logs
```bash
# View all logs
./scripts/logs.sh

# View specific service logs
./scripts/logs.sh backend
```

## 🔄 Workflow Examples

### Development Workflow
```bash
# 1. Start development environment
./scripts/dev.sh

# 2. Start file watching
./scripts/dev-watch.sh

# 3. View logs if needed
./scripts/logs.sh backend

# 4. Stop when done
./scripts/stop.sh
```

### Production Deployment Workflow
```bash
# 1. Deploy to production
./scripts/prod-deploy.sh

# 2. Monitor deployment
./scripts/logs.sh

# 3. Check service health
curl http://localhost/api/health

# 4. Access Grafana
open http://grafana.localhost
```

### Maintenance Workflow
```bash
# 1. Create backup
./scripts/backup.sh

# 2. View logs
./scripts/logs.sh

# 3. Stop services
./scripts/stop.sh

# 4. Reset if needed (CAUTION!)
./scripts/reset.sh
```

## 📚 Additional Resources

- **Docker Documentation:** https://docs.docker.com/
- **Docker Compose Documentation:** https://docs.docker.com/compose/
- **T-Force Project README:** ../README.md
- **Environment Configuration:** ../.env.example

## 🤝 Contributing

When adding new scripts:

1. **Follow the naming convention:** `script-name.sh`
2. **Add proper error handling:** `set -euo pipefail`
3. **Include logging functions:** Use the standard color scheme
4. **Add help documentation:** Include usage examples
5. **Make executable:** `chmod +x scripts/new-script.sh`
6. **Update this README:** Document the new script

## 📝 Script Template

```bash
#!/bin/bash

# Script Name
# Brief description of what the script does

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Main function
main() {
    log_info "Starting script..."
    # Your script logic here
    log_success "Script completed successfully!"
}

# Handle script interruption
trap 'log_error "Script interrupted by user"; exit 1' INT TERM

# Run main function
main "$@"
```

---

**Last Updated:** $(date)
**Version:** 1.0.0
