#!/bin/bash

# Colors for terminal output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Determine which docker-compose file to use
if [ -f docker-compose.dev.yml ] && docker compose -f docker-compose.dev.yml ps | grep -q "tforce-frontend"; then
  COMPOSE_FILE="docker-compose.dev.yml"
  echo -e "${YELLOW}Using development environment...${NC}"
else
  COMPOSE_FILE="docker-compose.yml"
  echo -e "${YELLOW}Using production environment...${NC}"
fi

# Check if .env.docker file exists
if [ ! -f .env.docker ]; then
  echo -e "${YELLOW}Warning: .env.docker file not found. Using default .env file.${NC}"
  ENV_FILE=".env"
else
  ENV_FILE=".env.docker"
fi

# Check if a specific service was requested
if [ "$1" == "frontend" ]; then
  echo -e "${YELLOW}Showing logs for frontend service...${NC}"
  docker compose -f $COMPOSE_FILE --env-file $ENV_FILE logs -f frontend
elif [ "$1" == "backend" ]; then
  echo -e "${YELLOW}Showing logs for backend service...${NC}"
  docker compose -f $COMPOSE_FILE --env-file $ENV_FILE logs -f backend
elif [ "$1" == "db" ] || [ "$1" == "database" ]; then
  echo -e "${YELLOW}Showing logs for database service...${NC}"
  docker compose -f $COMPOSE_FILE --env-file $ENV_FILE logs -f db
else
  # If no specific service, show usage or all logs
  if [ -z "$1" ]; then
    echo -e "${YELLOW}Showing logs for all services...${NC}"
    echo -e "${BLUE}Press Ctrl+C to exit${NC}\n"
    docker compose -f $COMPOSE_FILE --env-file $ENV_FILE logs -f
  else
    echo -e "${YELLOW}Usage:${NC}"
    echo -e "  ${GREEN}./scripts/logs.sh${NC}             - Show logs for all services"
    echo -e "  ${GREEN}./scripts/logs.sh frontend${NC}    - Show logs for frontend service"
    echo -e "  ${GREEN}./scripts/logs.sh backend${NC}     - Show logs for backend service"
    echo -e "  ${GREEN}./scripts/logs.sh db${NC}          - Show logs for database service"
  fi
fi