#!/bin/bash

# Colors for terminal output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if .env.docker file exists
if [ ! -f .env.docker ]; then
  echo -e "${YELLOW}Warning: .env.docker file not found. Using default .env file.${NC}"
  ENV_FILE=".env"
else
  ENV_FILE=".env.docker"
fi

# Check if we're stopping development or production
if [ "$1" == "dev" ] || [ "$1" == "development" ]; then
  echo -e "${YELLOW}Stopping development environment...${NC}"
  docker compose -f docker-compose.dev.yml --env-file $ENV_FILE down
  echo -e "${GREEN}Development environment stopped.${NC}"
elif [ "$1" == "prod" ] || [ "$1" == "production" ]; then
  echo -e "${YELLOW}Stopping production environment...${NC}"
  docker compose --env-file $ENV_FILE down
  echo -e "${GREEN}Production environment stopped.${NC}"
else
  # If no environment specified, try to stop both
  echo -e "${YELLOW}Stopping all environments...${NC}"
  
  # Try to stop development environment
  if [ -f docker-compose.dev.yml ]; then
    echo -e "${YELLOW}Stopping development environment...${NC}"
    docker compose -f docker-compose.dev.yml --env-file $ENV_FILE down
  fi
  
  # Try to stop production environment
  if [ -f docker-compose.yml ]; then
    echo -e "${YELLOW}Stopping production environment...${NC}"
    docker compose --env-file $ENV_FILE down
  fi
  
  echo -e "${GREEN}All environments stopped.${NC}"
fi