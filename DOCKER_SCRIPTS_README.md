# T-Force Docker Scripts Documentation

This document provides comprehensive documentation for all Docker-related scripts in the `scripts/` directory of the T-Force project. These scripts automate various aspects of development, deployment, testing, and maintenance workflows.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Environment Setup](#environment-setup)
- [Core Development Scripts](#core-development-scripts)
- [Production Scripts](#production-scripts)
- [Database Management Scripts](#database-management-scripts)
- [Utility Scripts](#utility-scripts)
- [Testing Scripts](#testing-scripts)
- [Maintenance Scripts](#maintenance-scripts)
- [Common Usage Patterns](#common-usage-patterns)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### Required Software
- **Docker**: Version 20.10 or higher
- **Docker Compose**: Version 2.0 or higher
- **Bash**: Version 4.0 or higher (macOS/Linux)
- **PostgreSQL Client Tools**: For database operations (optional)

### Required Files
- `.env` file (copy from `.env.example` and configure)
- `.env.docker` file (automatically created by scripts)
- `docker-compose.dev.yml` for development
- `docker-compose.yml` for production
- `docker-compose.prod.yml` for advanced production deployment

### Environment Variables
Ensure these variables are set in your `.env` file:
```bash
# Database
DATABASE_URL=postgres://postgres:postgres@db:5432/authforce
POSTGRES_USER=postgres
POSTGRES_PASSWORD=postgres
POSTGRES_DB=authforce

# Backend
HOST=0.0.0.0
PORT=8080
RUST_LOG=info

# Authentication
NEXTAUTH_URL=http://localhost
NEXTAUTH_SECRET=your-secure-secret-here

# OAuth (configure with your credentials)
GOOGLE_CLIENT_ID=your-google-client-id
GOOGLE_CLIENT_SECRET=your-google-client-secret
GITHUB_CLIENT_ID=your-github-client-id
GITHUB_CLIENT_SECRET=your-github-client-secret

# API URL
NEXT_PUBLIC_API_URL=http://localhost/api
```

## Core Development Scripts

### dev.sh
**Purpose**: Start the complete development environment with all services.

**Features**:
- Automatically creates `.env.docker` from `.env`
- Stops any running containers
- Builds and starts all services via Traefik
- Provides colored terminal output

**Usage**:
```bash
./scripts/dev.sh
```

**Services Started**:
- Frontend: `http://localhost` (via Traefik)
- Backend API: `http://localhost/api` (via Traefik)
- Database: `localhost:5433`
- Traefik Dashboard: `http://localhost:8080`

**Direct Port Access** (for debugging):
- Frontend: `http://localhost:3000`
- Backend: `http://localhost:8081`

### dev-watch.sh
**Purpose**: Start development environment with backend hot-reload capabilities.

**Features**:
- Automatic backend rebuilding on file changes
- Real-time log streaming
- Docker health checks
- Comprehensive startup feedback

**Usage**:
```bash
./scripts/dev-watch.sh
```

**Monitored Directories**:
- `backend/src/`
- `backend/migration/`
- `Cargo.toml` files

**Output**: Shows live logs and provides useful commands for development.

### restart-backend-watch.sh
**Purpose**: Restart only the backend service with watch mode enabled.

**Usage**:
```bash
./scripts/restart-backend-watch.sh
```

**Use Cases**:
- Backend configuration changes
- Dependency updates
- Debugging backend issues

## Production Scripts

### prod.sh
**Purpose**: Start the production environment with optimized settings.

**Features**:
- Uses production Docker configurations
- Optimized builds and caching
- No direct port exposure (Traefik only)
- Production-ready environment variables

**Usage**:
```bash
./scripts/prod.sh
```

**Access Points**:
- Application: `http://localhost` (via Traefik)
- API: `http://localhost/api` (via Traefik)

### prod-deploy.sh
**Purpose**: Complete production deployment with advanced features.

**Features**:
- SSL/TLS certificate management
- Security headers and rate limiting
- Health checks and monitoring
- Backup integration
- Zero-downtime deployment

**Usage**:
```bash
./scripts/prod-deploy.sh
```

**Requirements**:
- `.env.production` file
- `docker-compose.prod.yml`
- SSL certificates (if using HTTPS)

## Database Management Scripts

### migrate.sh
**Purpose**: Comprehensive database migration management.

**Commands**:
```bash
# Run all pending migrations
./scripts/migrate.sh up

# Revert the last migration
./scripts/migrate.sh down

# Drop all tables and rerun migrations
./scripts/migrate.sh fresh

# Check migration status
./scripts/migrate.sh status

# Generate a new migration
./scripts/migrate.sh generate create_users_table
```

**Prerequisites**:
- Database must be running
- `sea-orm-cli` installed in backend
- Valid `DATABASE_URL` in `.env`

### run-migrations.sh
**Purpose**: Simple migration runner using the migration binary.

**Usage**:
```bash
./scripts/run-migrations.sh
```

**Features**:
- Automatic database startup if needed
- Uses compiled migration binary
- Provides database status information

## Utility Scripts

### logs.sh
**Purpose**: View logs from Docker containers with filtering options.

**Usage**:
```bash
# View all service logs
./scripts/logs.sh

# View specific service logs
./scripts/logs.sh frontend
./scripts/logs.sh backend
./scripts/logs.sh db
```

**Features**:
- Automatic environment detection (dev/prod)
- Real-time log streaming
- Service-specific filtering
- Colored output

### stop.sh
**Purpose**: Stop Docker containers with environment-specific options.

**Usage**:
```bash
# Stop all environments
./scripts/stop.sh

# Stop specific environment
./scripts/stop.sh dev
./scripts/stop.sh prod
```

**Features**:
- Graceful container shutdown
- Environment-specific stopping
- Automatic cleanup

### docker.sh
**Purpose**: Unified Docker management tool with multiple commands.

**Usage**:
```bash
# Show help
./scripts/docker.sh help

# Start environments
./scripts/docker.sh dev
./scripts/docker.sh prod

# View logs
./scripts/docker.sh logs [service]

# Stop containers
./scripts/docker.sh stop [env]

# Show container status
./scripts/docker.sh status

# Reset environment
./scripts/docker.sh reset
```

## Testing Scripts

### test.sh
**Purpose**: Comprehensive system testing and validation.

**Tests Performed**:
1. Docker availability check
2. Environment variable validation
3. OAuth configuration verification
4. Database connectivity test
5. Service health checks

**Usage**:
```bash
./scripts/test.sh
```

**Exit Codes**:
- `0`: All tests passed
- `1`: Tests failed (check output for details)

### test_chat.sh
**Purpose**: Specific testing for chat system functionality.

**Features**:
- Backend health checks
- Frontend availability verification
- WebSocket endpoint testing
- Chat system integration tests

**Usage**:
```bash
./scripts/test_chat.sh
```

## Maintenance Scripts

### backup.sh
**Purpose**: Automated database and application backup.

**Features**:
- Compressed database dumps
- Backup manifest generation
- Automatic cleanup (30-day retention)
- Size and status reporting

**Usage**:
```bash
./scripts/backup.sh
```

**Output Location**: `/backups/` directory
**Files Created**:
- `tforce_db_YYYYMMDD_HHMMSS.sql.gz`
- `backup_manifest_YYYYMMDD_HHMMSS.json`

### reset.sh
**Purpose**: Complete Docker environment reset.

**⚠️ WARNING**: This script removes ALL containers, volumes, and images!

**Usage**:
```bash
./scripts/reset.sh
```

**Actions Performed**:
1. Stop all containers
2. Remove containers and volumes
3. Remove project-specific images
4. Clean up dangling images

### clear-auth-token.sh
**Purpose**: Help users clear invalid authentication tokens.

**Usage**:
```bash
./scripts/clear-auth-token.sh
```

**Features**:
- Provides browser-based token clearing instructions
- Shows current database users
- Offers console commands for token removal

### fix-room-memberships.sh
**Purpose**: Fix chat room membership issues.

**Usage**:
```bash
./scripts/fix-room-memberships.sh
```

**Features**:
- Identifies rooms without creator memberships
- Automatically adds missing memberships
- Provides detailed progress reporting

### setup-shadcn.sh
**Purpose**: Set up shadcn UI components for the frontend.

**Usage**:
```bash
./scripts/setup-shadcn.sh
```

**Features**:
- Installs shadcn and dependencies
- Initializes component library
- Adds commonly used components

## Common Usage Patterns

### Quick Development Start
```bash
# 1. Set up environment
cp .env.example .env
# Edit .env with your configuration

# 2. Start development environment
./scripts/dev.sh

# 3. View logs (in another terminal)
./scripts/logs.sh
```

### Development with Hot Reload
```bash
# Start with watch mode
./scripts/dev-watch.sh

# Make changes to backend code
# Backend automatically rebuilds

# Restart only backend if needed
./scripts/restart-backend-watch.sh
```

### Database Operations
```bash
# Check migration status
./scripts/migrate.sh status

# Run pending migrations
./scripts/migrate.sh up

# Create new migration
./scripts/migrate.sh generate add_user_preferences
```

### Production Deployment
```bash
# 1. Prepare production environment
cp env.production.template .env.production
# Configure production variables

# 2. Deploy
./scripts/prod-deploy.sh

# 3. Monitor
./scripts/logs.sh
```

### Testing and Validation
```bash
# Run comprehensive tests
./scripts/test.sh

# Test specific functionality
./scripts/test_chat.sh

# Check system status
./scripts/docker.sh status
```

### Maintenance Tasks
```bash
# Create backup
./scripts/backup.sh

# Clean up environment
./scripts/stop.sh
./scripts/reset.sh  # Use with caution!

# Fix common issues
./scripts/clear-auth-token.sh
./scripts/fix-room-memberships.sh
```

## Troubleshooting

### Common Issues

#### "Docker is not running"
```bash
# Start Docker Desktop or Docker daemon
sudo systemctl start docker  # Linux
# or start Docker Desktop app
```

#### "Environment file not found"
```bash
# Create from template
cp .env.example .env
# Edit with your configuration
```

#### "Database connection failed"
```bash
# Check if database is running
./scripts/logs.sh db

# Restart database
./scripts/stop.sh
./scripts/dev.sh
```

#### "Port already in use"
```bash
# Stop conflicting services
./scripts/stop.sh

# Or use different ports in docker-compose files
```

#### "Migration failed"
```bash
# Check database status
./scripts/migrate.sh status

# Reset database (⚠️ data loss)
./scripts/migrate.sh fresh
```

### Debug Commands

```bash
# Check container status
docker ps -a

# View specific container logs
docker logs tforce-backend

# Execute commands in container
docker exec -it tforce-backend bash

# Check network connectivity
docker network ls
docker network inspect tforce-network
```

### Performance Issues

```bash
# Check resource usage
docker stats

# Clean up unused resources
docker system prune

# Rebuild without cache
./scripts/stop.sh
docker-compose -f docker-compose.dev.yml build --no-cache
./scripts/dev.sh
```

## Script Dependencies

### Internal Dependencies
- `dev.sh` → `stop.sh` (implicit)
- `prod-deploy.sh` → `backup.sh`
- `migrate.sh` → Database container
- `logs.sh` → Running containers

### External Dependencies
- All scripts require Docker and Docker Compose
- Migration scripts require `sea-orm-cli`
- Backup scripts require `pg_dump`
- Test scripts require `curl`

## Security Considerations

- Never commit `.env` files to version control
- Use strong passwords for production databases
- Regularly update Docker images
- Review and rotate OAuth credentials
- Monitor access logs in production
- Use HTTPS in production environments

## Contributing

When adding new scripts:
1. Follow the existing naming convention
2. Include colored output for better UX
3. Add comprehensive error handling
4. Document all parameters and options
5. Update this README with new script documentation

---

**Last Updated**: $(date)
**Project**: T-Force
**Version**: 1.0.0