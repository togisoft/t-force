#!/bin/bash

# T-Force Development Watch Script
# Monitors file changes and restarts services automatically

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

# Check if development environment is running
check_dev_environment() {
    log_info "Checking development environment..."
    
    if ! docker compose -f "$COMPOSE_FILE" ps | grep -q "Up"; then
        log_error "Development environment is not running!"
        log_info "Please run './scripts/dev.sh' first to start the development environment."
        exit 1
    fi
    
    log_success "Development environment is running"
}

# Watch backend changes
watch_backend() {
    log_info "Watching backend changes..."
    
    # Watch for Rust file changes
    if command -v fswatch > /dev/null 2>&1; then
        log_info "Using fswatch to monitor backend changes..."
        fswatch -o backend/src/ | while read; do
            log_info "Backend files changed, restarting backend..."
            docker compose -f "$COMPOSE_FILE" restart backend
        done
    else
        log_warning "fswatch not installed. Install with: brew install fswatch"
        log_info "Falling back to manual restart mode..."
        log_info "Press Ctrl+C to stop watching"
        while true; do
            sleep 5
        done
    fi
}

# Watch frontend changes
watch_frontend() {
    log_info "Watching frontend changes..."
    
    # Watch for Next.js file changes
    if command -v fswatch > /dev/null 2>&1; then
        log_info "Using fswatch to monitor frontend changes..."
        fswatch -o frontend/ | while read; do
            log_info "Frontend files changed, restarting frontend..."
            docker compose -f "$COMPOSE_FILE" restart frontend
        done
    else
        log_warning "fswatch not installed. Install with: brew install fswatch"
        log_info "Falling back to manual restart mode..."
        log_info "Press Ctrl+C to stop watching"
        while true; do
            sleep 5
        done
    fi
}

# Watch all changes
watch_all() {
    log_info "Watching all changes..."
    
    if command -v fswatch > /dev/null 2>&1; then
        log_info "Using fswatch to monitor all changes..."
        fswatch -o backend/src/ frontend/ | while read; do
            log_info "Files changed, restarting services..."
            docker compose -f "$COMPOSE_FILE" restart
        done
    else
        log_warning "fswatch not installed. Install with: brew install fswatch"
        log_info "Falling back to manual restart mode..."
        log_info "Press Ctrl+C to stop watching"
        while true; do
            sleep 5
        done
    fi
}

# Show usage
show_usage() {
    echo "T-Force Development Watch"
    echo "========================"
    echo ""
    echo "Usage:"
    echo "  ./scripts/dev-watch.sh [backend|frontend|all]"
    echo ""
    echo "Options:"
    echo "  backend   - Watch only backend changes"
    echo "  frontend  - Watch only frontend changes"
    echo "  all       - Watch all changes (default)"
    echo ""
    echo "Requirements:"
    echo "  - fswatch (install with: brew install fswatch)"
    echo "  - Development environment running (./scripts/dev.sh)"
}

# Main function
main() {
    case "${1:-all}" in
        "backend")
            check_dev_environment
            watch_backend
            ;;
        "frontend")
            check_dev_environment
            watch_frontend
            ;;
        "all")
            check_dev_environment
            watch_all
            ;;
        "help"|"-h"|"--help")
            show_usage
            ;;
        *)
            log_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
}

# Handle script interruption
trap 'log_info "Watch mode stopped by user"; exit 0' INT TERM

# Run main function
main "$@"
