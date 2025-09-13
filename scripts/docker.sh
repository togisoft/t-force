#!/bin/bash

# Colors for terminal output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Display help message
show_help() {
  echo -e "${BLUE}T-Force Docker Management Tool${NC}"
  echo -e "Usage: ./scripts/docker.sh [command] [options]"
  echo -e ""
  echo -e "Commands:"
  echo -e "  ${GREEN}dev${NC}                Start development environment"
  echo -e "  ${GREEN}prod${NC}               Start production environment"
  echo -e "  ${GREEN}stop${NC} [env]         Stop containers (env: dev, prod, or all)"
  echo -e "  ${GREEN}logs${NC} [service]     View logs (service: frontend, backend, db, or all)"
  echo -e "  ${GREEN}reset${NC}              Reset Docker environment (remove all containers, volumes, and images)"
  echo -e "  ${GREEN}status${NC}             Show status of running containers"
  echo -e "  ${GREEN}help${NC}               Show this help message"
  echo -e ""
  echo -e "Examples:"
  echo -e "  ${YELLOW}./scripts/docker.sh dev${NC}           Start development environment"
  echo -e "  ${YELLOW}./scripts/docker.sh logs frontend${NC} View frontend logs"
  echo -e "  ${YELLOW}./scripts/docker.sh stop all${NC}      Stop all containers"
}

# Check if .env file exists
check_env_file() {
  if [ ! -f .env ]; then
    echo -e "${RED}Error: .env file not found!${NC}"
    echo -e "Please create a .env file based on .env.example"
    exit 1
  fi
}

# Check if .env.docker file exists, create it if not
ensure_docker_env_file() {
  if [ ! -f .env.docker ]; then
    echo -e "${YELLOW}Creating .env.docker file from .env...${NC}"
    cp .env .env.docker
    echo -e "\n# For container-to-container communication (used by Docker)" >> .env.docker
    echo -e "NEXT_PUBLIC_INTERNAL_API_URL=http://backend:8080" >> .env.docker
  fi
}

# Start development environment
start_dev() {
  echo -e "${YELLOW}Starting T-Force in development mode...${NC}"
  check_env_file
  ensure_docker_env_file
  
  # Stop any running containers
  echo -e "${YELLOW}Stopping any running containers...${NC}"
  docker compose -f docker-compose.dev.yml down
  
  # Copy Cargo.lock to backend directory for Docker build
  echo -e "${YELLOW}Copying Cargo.lock to backend directory...${NC}"
  cp Cargo.lock backend/ 2>/dev/null || true
  
  # Build and start the containers
  echo -e "${YELLOW}Building and starting containers...${NC}"
  docker compose -f docker-compose.dev.yml --env-file .env.docker up --build -d
  
  # Show status
  echo -e "${GREEN}T-Force development environment is running!${NC}"
  echo -e "${YELLOW}Frontend:${NC} http://localhost:3000"
  echo -e "${YELLOW}Backend:${NC} http://localhost:8080"
}

# Start production environment
start_prod() {
  echo -e "${YELLOW}Starting T-Force in production mode...${NC}"
  check_env_file
  ensure_docker_env_file
  
  # Stop any running containers
  echo -e "${YELLOW}Stopping any running containers...${NC}"
  docker compose down
  
  # Copy Cargo.lock to backend directory for Docker build
  echo -e "${YELLOW}Copying Cargo.lock to backend directory...${NC}"
  cp Cargo.lock backend/ 2>/dev/null || true
  
  # Build and start the containers
  echo -e "${YELLOW}Building and starting containers in production mode...${NC}"
  docker compose --env-file .env.docker up --build -d
  
  # Show status
  echo -e "${GREEN}T-Force production environment is running!${NC}"
  echo -e "${YELLOW}Frontend:${NC} http://localhost:3000"
  echo -e "${YELLOW}Backend:${NC} http://localhost:8080"
}

# Stop containers
stop_containers() {
  ensure_docker_env_file
  
  if [ "$1" == "dev" ] || [ "$1" == "development" ]; then
    echo -e "${YELLOW}Stopping development environment...${NC}"
    docker compose -f docker-compose.dev.yml --env-file .env.docker down
    echo -e "${GREEN}Development environment stopped.${NC}"
  elif [ "$1" == "prod" ] || [ "$1" == "production" ]; then
    echo -e "${YELLOW}Stopping production environment...${NC}"
    docker compose --env-file .env.docker down
    echo -e "${GREEN}Production environment stopped.${NC}"
  else
    # If no environment specified, try to stop both
    echo -e "${YELLOW}Stopping all environments...${NC}"
    
    # Try to stop development environment
    if [ -f docker-compose.dev.yml ]; then
      echo -e "${YELLOW}Stopping development environment...${NC}"
      docker compose -f docker-compose.dev.yml --env-file .env.docker down
    fi
    
    # Try to stop production environment
    if [ -f docker-compose.yml ]; then
      echo -e "${YELLOW}Stopping production environment...${NC}"
      docker compose --env-file .env.docker down
    fi
    
    echo -e "${GREEN}All environments stopped.${NC}"
  fi
}

# View logs
view_logs() {
  ensure_docker_env_file
  
  # Determine which docker-compose file to use
  if [ -f docker-compose.dev.yml ] && docker compose -f docker-compose.dev.yml ps | grep -q "tforce-frontend"; then
    COMPOSE_FILE="docker-compose.dev.yml"
    echo -e "${YELLOW}Using development environment...${NC}"
  else
    COMPOSE_FILE="docker-compose.yml"
    echo -e "${YELLOW}Using production environment...${NC}"
  fi
  
  # Check if a specific service was requested
  if [ "$1" == "frontend" ]; then
    echo -e "${YELLOW}Showing logs for frontend service...${NC}"
    docker compose -f $COMPOSE_FILE --env-file .env.docker logs -f frontend
  elif [ "$1" == "backend" ]; then
    echo -e "${YELLOW}Showing logs for backend service...${NC}"
    docker compose -f $COMPOSE_FILE --env-file .env.docker logs -f backend
  elif [ "$1" == "db" ] || [ "$1" == "database" ]; then
    echo -e "${YELLOW}Showing logs for database service...${NC}"
    docker compose -f $COMPOSE_FILE --env-file .env.docker logs -f db
  else
    # If no specific service, show all logs
    echo -e "${YELLOW}Showing logs for all services...${NC}"
    echo -e "${BLUE}Press Ctrl+C to exit${NC}\n"
    docker compose -f $COMPOSE_FILE --env-file .env.docker logs -f
  fi
}

# Reset Docker environment
reset_environment() {
  echo -e "${YELLOW}T-Force Docker Environment Reset Tool${NC}"
  echo -e "${RED}WARNING: This will remove all containers, volumes, and images related to this project.${NC}"
  echo -e "${RED}All data will be lost!${NC}"
  echo -e "${BLUE}Press Ctrl+C now to cancel${NC}"
  echo ""
  read -p "Are you sure you want to continue? (y/n): " -n 1 -r
  echo ""
  
  if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Stop all containers
    echo -e "${YELLOW}Stopping all containers...${NC}"
    stop_containers all
    
    # Remove all containers, networks, and volumes
    echo -e "${YELLOW}Removing all containers, networks, and volumes...${NC}"
    docker compose -f docker-compose.dev.yml down -v --remove-orphans
    docker compose down -v --remove-orphans
    
    # Remove all images related to the project
    echo -e "${YELLOW}Removing all project images...${NC}"
    docker images | grep tforce | awk '{print $3}' | xargs -r docker rmi -f
    
    # Clean up any dangling images
    echo -e "${YELLOW}Cleaning up dangling images...${NC}"
    docker image prune -f
    
    echo -e "${GREEN}Reset complete!${NC}"
    echo -e "You can now run ${YELLOW}./scripts/docker.sh dev${NC} or ${YELLOW}./scripts/docker.sh prod${NC} to start fresh."
  else
    echo -e "${BLUE}Reset cancelled.${NC}"
  fi
}

# Show status of running containers
show_status() {
  echo -e "${YELLOW}Development Environment:${NC}"
  docker compose -f docker-compose.dev.yml ps
  
  echo -e "\n${YELLOW}Production Environment:${NC}"
  docker compose ps
}

# Main script logic
case "$1" in
  dev|development)
    start_dev
    ;;
  prod|production)
    start_prod
    ;;
  stop)
    stop_containers "$2"
    ;;
  logs)
    view_logs "$2"
    ;;
  reset)
    reset_environment
    ;;
  status)
    show_status
    ;;
  help|--help|-h)
    show_help
    ;;
  *)
    echo -e "${RED}Error: Unknown command '$1'${NC}"
    show_help
    exit 1
    ;;
esac

exit 0