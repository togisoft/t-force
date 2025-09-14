#!/bin/bash

# T-Force Production Deployment Script
# Modern, clean deployment automation for production environment

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
COMPOSE_FILE="docker-compose.prod.yml"
ENV_FILE=".env.prod"
HEALTH_CHECK_TIMEOUT=60
SERVICE_STARTUP_DELAY=10

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

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if Docker is running
    if ! docker info > /dev/null 2>&1; then
        log_error "Docker is not running. Please start Docker and try again."
        exit 1
    fi
    
    # Check if docker compose is available
    if ! command -v docker > /dev/null 2>&1; then
        log_error "Docker Compose is not installed or not in PATH."
        exit 1
    fi
    
    # Check if .env.prod exists
    if [ ! -f "$ENV_FILE" ]; then
        log_error ".env.prod file not found!"
        log_info "Please create .env.prod file with your production environment variables."
        exit 1
    fi
    
    # Check if docker-compose.prod.yml exists
    if [ ! -f "$COMPOSE_FILE" ]; then
        log_error "docker-compose.prod.yml file not found!"
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

# Load and validate environment variables
load_environment() {
    log_info "Loading environment variables from $ENV_FILE..."
    
    # Load environment variables
    set -a
    source "$ENV_FILE"
    set +a
    
    # Required environment variables
    local required_vars=(
        "POSTGRES_PASSWORD"
        "NEXTAUTH_SECRET" 
        "JWT_SECRET"
        "POSTGRES_USER"
        "POSTGRES_DB"
    )
    
    # Check required variables
    local missing_vars=()
    for var in "${required_vars[@]}"; do
        if [ -z "${!var:-}" ]; then
            missing_vars+=("$var")
        fi
    done
    
    if [ ${#missing_vars[@]} -ne 0 ]; then
        log_error "Missing required environment variables:"
        for var in "${missing_vars[@]}"; do
            echo "  - $var"
        done
        exit 1
    fi
    
    log_success "Environment variables loaded and validated"
}

# Clean up existing deployment
cleanup_existing() {
    log_info "Cleaning up existing deployment..."
    
    # Stop and remove containers
    if docker compose -f "$COMPOSE_FILE" ps -q | grep -q .; then
        log_info "Stopping existing containers..."
        docker compose -f "$COMPOSE_FILE" down --remove-orphans
    fi
    
    # Remove old images (optional - uncomment if you want to force rebuild)
    # log_info "Removing old images..."
    # docker compose -f "$COMPOSE_FILE" down --rmi all --volumes --remove-orphans
    
    log_success "Cleanup completed"
}

# Deploy services
deploy_services() {
    log_info "Deploying services..."
    
    # Build and start services
    docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" up --build -d
    
    log_success "Services deployment initiated"
}

# Wait for service to be healthy
wait_for_service() {
    local service_name=$1
    local health_check_cmd=$2
    local timeout=${3:-$HEALTH_CHECK_TIMEOUT}
    
    log_info "Waiting for $service_name to be healthy (timeout: ${timeout}s)..."
    
    local elapsed=0
    while [ $elapsed -lt $timeout ]; do
        if eval "$health_check_cmd" > /dev/null 2>&1; then
            log_success "$service_name is healthy"
            return 0
        fi
        
        sleep 2
        elapsed=$((elapsed + 2))
        echo -n "."
    done
    
    echo ""
    log_error "$service_name failed to become healthy within ${timeout}s"
    return 1
}

# Health checks
run_health_checks() {
    log_info "Running health checks..."
    
    # Wait for services to start
    log_info "Waiting ${SERVICE_STARTUP_DELAY}s for services to start..."
    sleep $SERVICE_STARTUP_DELAY
    
    # Check database
    wait_for_service "Database" "docker compose -f $COMPOSE_FILE exec -T db pg_isready -U $POSTGRES_USER -d $POSTGRES_DB"
    
    
    # Check backend
    wait_for_service "Backend" "curl -f http://localhost/health"
    
    log_success "All health checks passed"
}

# Run database migrations
run_migrations() {
    log_info "Running database migrations..."
    
    # Run migrations in a separate container to avoid port conflicts
    if docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" run --rm backend ./tforce migrate; then
        log_success "Database migrations completed"
    else
        log_error "Database migrations failed"
        exit 1
    fi
}

# Display deployment summary
show_summary() {
    log_success "Production deployment completed successfully!"
    echo ""
    echo "Service Status:"
    docker compose -f "$COMPOSE_FILE" ps
    echo ""
    echo "Service URLs:"
    echo "  - Application: http://localhost"
    echo "  - Traefik Dashboard: http://localhost:8080/dashboard/"
    echo "  - Health Check: http://localhost/health"
    echo ""
    echo "Useful Commands:"
    echo "  - View logs: docker compose -f $COMPOSE_FILE logs -f"
    echo "  - Stop services: docker compose -f $COMPOSE_FILE down"
    echo "  - Restart services: docker compose -f $COMPOSE_FILE restart"
    echo "  - Update services: docker compose -f $COMPOSE_FILE pull && docker compose -f $COMPOSE_FILE up -d"
    echo "  - WebSocket Test: open ws-latency-test.html"
}

# Main deployment function
main() {
    echo "T-Force Production Deployment"
    echo "=============================="
    echo ""
    
    check_prerequisites
    load_environment
    cleanup_existing
    deploy_services
    run_health_checks
    run_migrations
    show_summary
}

# Handle script interruption
trap 'log_error "Deployment interrupted by user"; exit 1' INT TERM

# Run main function
main "$@"