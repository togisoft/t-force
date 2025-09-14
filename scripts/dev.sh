#!/bin/bash

# T-Force Development Environment Script
# Clean, modern development setup

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
COMPOSE_FILE="docker-compose.dev.yml"
ENV_FILE=".env.dev"

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
    
    # Check if .env.dev exists
    if [ ! -f "$ENV_FILE" ]; then
        log_warning ".env.dev file not found!"
        log_info "Creating .env.dev from .env.example..."
        if [ -f ".env.example" ]; then
            cp .env.example "$ENV_FILE"
            log_success ".env.dev created from .env.example"
        else
            log_error ".env.example file not found!"
            exit 1
        fi
    fi
    
    # Check if docker-compose.dev.yml exists
    if [ ! -f "$COMPOSE_FILE" ]; then
        log_error "docker-compose.dev.yml file not found!"
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

# Clean up existing development environment
cleanup_dev() {
    log_info "Cleaning up existing development environment..."
    
    # Stop and remove containers
    if docker compose -f "$COMPOSE_FILE" ps -q | grep -q .; then
        log_info "Stopping existing development containers..."
        docker compose -f "$COMPOSE_FILE" down --remove-orphans
    fi
    
    log_success "Development cleanup completed"
}

# Start development environment
start_dev() {
    log_info "Starting development environment..."
    
    # Start services
    docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" up -d
    
    log_success "Development environment started"
}

# Wait for services to be ready
wait_for_services() {
    log_info "Waiting for services to be ready..."
    sleep 10
    
    # Check if services are running
    if docker compose -f "$COMPOSE_FILE" ps | grep -q "Up"; then
        log_success "Services are running"
    else
        log_warning "Some services may not be ready yet"
    fi
}

# Display development summary
show_summary() {
    log_success "Development environment is ready!"
    echo ""
    echo "Service URLs:"
    echo "  - Frontend: http://localhost:3000"
    echo "  - Backend: http://localhost:8080"
    echo "  - Database: localhost:5432"
    echo "  - Redis: localhost:6379"
    echo ""
    echo "Useful Commands:"
    echo "  - View logs: docker compose -f $COMPOSE_FILE logs -f"
    echo "  - Stop services: docker compose -f $COMPOSE_FILE down"
    echo "  - Restart services: docker compose -f $COMPOSE_FILE restart"
    echo "  - Watch mode: ./scripts/dev-watch.sh"
}

# Main function
main() {
    echo "T-Force Development Environment"
    echo "==============================="
    echo ""
    
    check_prerequisites
    cleanup_dev
    start_dev
    wait_for_services
    show_summary
}

# Handle script interruption
trap 'log_error "Development setup interrupted by user"; exit 1' INT TERM

# Run main function
main "$@"
