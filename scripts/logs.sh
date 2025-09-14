#!/bin/bash

# T-Force Logs Script
# Modern, clean log viewing for development and production

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

# Determine which environment to use
detect_environment() {
    if [ -f "docker-compose.dev.yml" ] && docker compose -f docker-compose.dev.yml ps | grep -q "tforce-frontend"; then
        COMPOSE_FILE="docker-compose.dev.yml"
        ENV_FILE=".env.dev"
        ENV_TYPE="development"
    elif [ -f "docker-compose.prod.yml" ] && docker compose -f docker-compose.prod.yml ps | grep -q "tforce-backend-prod"; then
        COMPOSE_FILE="docker-compose.prod.yml"
        ENV_FILE=".env.prod"
        ENV_TYPE="production"
    else
        COMPOSE_FILE="docker-compose.yml"
        ENV_FILE=".env"
        ENV_TYPE="default"
    fi
    
    log_info "Using $ENV_TYPE environment ($COMPOSE_FILE)"
}

# Show logs for specific service
show_service_logs() {
    local service=$1
    log_info "Showing logs for $service service..."
    echo -e "${BLUE}Press Ctrl+C to exit${NC}\n"
    docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" logs -f "$service"
}

# Show all logs
show_all_logs() {
    log_info "Showing logs for all services..."
    echo -e "${BLUE}Press Ctrl+C to exit${NC}\n"
    docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" logs -f
}

# Show usage
show_usage() {
    echo "T-Force Logs"
    echo "============"
    echo ""
    echo "Usage:"
    echo "  ./scripts/logs.sh [service]"
    echo ""
    echo "Services:"
    echo "  frontend  - Show frontend logs"
    echo "  backend   - Show backend logs"
    echo "  db        - Show database logs"
    echo "  traefik   - Show Traefik logs"
    echo "  grafana   - Show Grafana logs"
    echo "  prometheus - Show Prometheus logs"
    echo ""
    echo "Examples:"
    echo "  ./scripts/logs.sh           - Show all logs"
    echo "  ./scripts/logs.sh backend   - Show backend logs only"
    echo "  ./scripts/logs.sh frontend  - Show frontend logs only"
}

# Main function
main() {
    detect_environment
    
    case "${1:-all}" in
        "frontend")
            show_service_logs "frontend"
            ;;
        "backend")
            show_service_logs "backend"
            ;;
        "db"|"database")
            show_service_logs "db"
            ;;
        "traefik")
            show_service_logs "traefik"
            ;;
        "grafana")
            show_service_logs "grafana"
            ;;
        "prometheus")
            show_service_logs "prometheus"
            ;;
        "all")
            show_all_logs
            ;;
        "help"|"-h"|"--help")
            show_usage
            ;;
        *)
            log_error "Unknown service: $1"
            show_usage
            exit 1
            ;;
    esac
}

# Handle script interruption
trap 'log_info "Log viewing stopped by user"; exit 0' INT TERM

# Run main function
main "$@"