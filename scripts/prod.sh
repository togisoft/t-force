#!/bin/bash

# Colors for terminal output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Starting T-Force in production mode...${NC}"

# Check if .env file exists
if [ ! -f .env ]; then
  echo -e "${RED}Error: .env file not found!${NC}"
  echo -e "Please create a .env file based on .env.example"
  exit 1
fi

# Check if .env.docker file exists, create it if not
if [ ! -f .env.docker ]; then
  echo -e "${YELLOW}Creating .env.docker file from .env...${NC}"
  cp .env .env.docker
  echo -e "\n# For container-to-container communication (used by Docker)" >> .env.docker
  echo -e "NEXT_PUBLIC_INTERNAL_API_URL=http://backend:8080" >> .env.docker
fi

# Stop any running containers
echo -e "${YELLOW}Stopping any running containers...${NC}"
docker compose down

# Build and start the containers
echo -e "${YELLOW}Building and starting containers in production mode...${NC}"
docker compose --env-file .env.docker up --build -d

# Show status
echo -e "${GREEN}T-Force production environment is running!${NC}"
echo -e "${YELLOW}Frontend:${NC} http://localhost:3000"
echo -e "${YELLOW}Backend:${NC} http://localhost:8080"
echo -e "\nUse ${YELLOW}scripts/logs.sh${NC} to view logs"
echo -e "Use ${YELLOW}scripts/stop.sh${NC} to stop the services"