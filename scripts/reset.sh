#!/bin/bash

# Colors for terminal output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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
  ./scripts/stop.sh
  
  # Remove all containers, networks, and volumes
  echo -e "${YELLOW}Removing all containers, networks, and volumes...${NC}"
  docker-compose -f docker-compose.dev.yml down -v --remove-orphans
  docker-compose down -v --remove-orphans
  
  # Remove all images related to the project
  echo -e "${YELLOW}Removing all project images...${NC}"
  docker images | grep tforce | awk '{print $3}' | xargs -r docker rmi -f
  
  # Clean up any dangling images
  echo -e "${YELLOW}Cleaning up dangling images...${NC}"
  docker image prune -f
  
  echo -e "${GREEN}Reset complete!${NC}"
  echo -e "You can now run ${YELLOW}./scripts/dev.sh${NC} or ${YELLOW}./scripts/prod.sh${NC} to start fresh."
else
  echo -e "${BLUE}Reset cancelled.${NC}"
fi