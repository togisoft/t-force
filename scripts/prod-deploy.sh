#!/bin/bash

# T-Force Production Deployment Script
# This script handles the complete production deployment process

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
COMPOSE_FILE="docker-compose.prod.yml"
ENV_FILE=".env.production"

echo -e "${BLUE}🚀 T-Force Production Deployment${NC}"
echo -e "${YELLOW}Date:${NC} $(date)"
echo ""

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
echo -e "${BLUE}🔍 Checking prerequisites...${NC}"

if ! command_exists docker; then
    echo -e "${RED}❌ Docker is not installed!${NC}"
    exit 1
fi

if ! command_exists docker-compose; then
    echo -e "${RED}❌ Docker Compose is not installed!${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Prerequisites check passed${NC}"

# Check environment file
if [ ! -f "$ENV_FILE" ]; then
    echo -e "${RED}❌ Environment file not found: $ENV_FILE${NC}"
    echo -e "${YELLOW}Please copy env.production.template to $ENV_FILE and configure it${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Environment file found${NC}"

# Load environment variables
echo -e "${BLUE}📋 Loading environment variables...${NC}"
source "$ENV_FILE"

# Validate required environment variables
echo -e "${BLUE}🔍 Validating environment variables...${NC}"

required_vars=(
    "POSTGRES_USER"
    "POSTGRES_PASSWORD"
    "POSTGRES_DB"
    "NEXTAUTH_SECRET"
    "JWT_SECRET"
    "REDIS_PASSWORD"
    "GRAFANA_PASSWORD"
)

for var in "${required_vars[@]}"; do
    if [ -z "${!var}" ]; then
        echo -e "${RED}❌ Missing required environment variable: $var${NC}"
        exit 1
    fi
done

echo -e "${GREEN}✅ Environment variables validated${NC}"

# Create necessary directories
echo -e "${BLUE}📁 Creating necessary directories...${NC}"
mkdir -p backups logs uploads monitoring/traefik

# Set proper permissions
chmod 755 backups logs uploads
chmod 600 "$ENV_FILE"

echo -e "${GREEN}✅ Directories created${NC}"

# Stop existing containers
echo -e "${BLUE}🛑 Stopping existing containers...${NC}"
docker-compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" down --remove-orphans

# Remove old images
echo -e "${BLUE}🧹 Cleaning up old images...${NC}"
docker image prune -f
docker system prune -f

# Pull latest images
echo -e "${BLUE}📥 Pulling latest images...${NC}"
docker-compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" pull

# Build and start services
echo -e "${BLUE}🔨 Building and starting services...${NC}"
docker-compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" up -d --build

# Wait for services to be healthy
echo -e "${BLUE}⏳ Waiting for services to be healthy...${NC}"
sleep 30

# Check service health
echo -e "${BLUE}🏥 Checking service health...${NC}"
services=("db" "redis" "backend" "frontend" "prometheus" "grafana")

for service in "${services[@]}"; do
    if docker-compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" ps "$service" | grep -q "Up"; then
        echo -e "${GREEN}✅ $service is running${NC}"
    else
        echo -e "${RED}❌ $service is not running properly${NC}"
        echo -e "${YELLOW}Check logs with: docker-compose -f $COMPOSE_FILE --env-file $ENV_FILE logs $service${NC}"
    fi
done

# Run database migrations
echo -e "${BLUE}🗄️ Running database migrations...${NC}"
docker-compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" exec -T backend cargo run --bin migration up

# Create initial backup
echo -e "${BLUE}💾 Creating initial backup...${NC}"
docker-compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" run --rm backup

# Show deployment summary
echo -e "${BLUE}📋 Deployment Summary:${NC}"
echo -e "${YELLOW}Application URL:${NC} https://yourdomain.com"
echo -e "${YELLOW}Traefik Dashboard:${NC} https://traefik.yourdomain.com (admin/password)"
echo -e "${YELLOW}Grafana Dashboard:${NC} https://grafana.yourdomain.com (admin/your_grafana_password)"
echo -e "${YELLOW}Prometheus:${NC} https://monitoring.yourdomain.com (admin/password)"

echo -e "${GREEN}🎉 T-Force production deployment completed successfully!${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo -e "1. Update your DNS to point yourdomain.com to this server"
echo -e "2. Configure SSL certificates (automatic with Let's Encrypt)"
echo -e "3. Set up monitoring alerts in Grafana"
echo -e "4. Configure automated backups"
echo -e "5. Set up log aggregation" 