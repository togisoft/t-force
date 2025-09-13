#!/bin/bash

# Colors for terminal output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Running database migrations for T-Force...${NC}"

# Check if .env file exists
if [ ! -f .env ]; then
  echo -e "${RED}Error: .env file not found!${NC}"
  echo -e "Please create a .env file based on .env.example"
  exit 1
fi

# Load environment variables
source .env

# Check if the database is running
echo -e "${YELLOW}Checking database connection...${NC}"
if ! pg_isready -h $(echo $DATABASE_URL | sed -E 's/.*@([^:]+):.*/\1/') -p $(echo $DATABASE_URL | sed -E 's/.*:([0-9]+)\/.*/\1/'); then
  echo -e "${RED}Error: Cannot connect to database!${NC}"
  echo -e "Make sure the database is running. You can start it with: scripts/dev.sh"
  exit 1
fi

# Change to the backend directory
cd backend

# Run migrations
echo -e "${YELLOW}Running migrations...${NC}"
if [ "$1" = "up" ]; then
  sea-orm-cli migrate up
elif [ "$1" = "down" ]; then
  sea-orm-cli migrate down
elif [ "$1" = "fresh" ]; then
  sea-orm-cli migrate fresh
elif [ "$1" = "status" ]; then
  sea-orm-cli migrate status
elif [ "$1" = "generate" ]; then
  if [ -z "$2" ]; then
    echo -e "${RED}Error: Migration name is required for generate command!${NC}"
    echo -e "Usage: scripts/migrate.sh generate <migration_name>"
    exit 1
  fi
  sea-orm-cli migrate generate "$2"
else
  echo -e "${YELLOW}Available commands:${NC}"
  echo -e "  ${GREEN}scripts/migrate.sh up${NC} - Run all pending migrations"
  echo -e "  ${GREEN}scripts/migrate.sh down${NC} - Revert the last migration"
  echo -e "  ${GREEN}scripts/migrate.sh fresh${NC} - Drop all tables and rerun all migrations"
  echo -e "  ${GREEN}scripts/migrate.sh status${NC} - Check the status of migrations"
  echo -e "  ${GREEN}scripts/migrate.sh generate <name>${NC} - Generate a new migration"
  exit 0
fi

# Check if the command was successful
if [ $? -eq 0 ]; then
  echo -e "${GREEN}Migration command completed successfully!${NC}"
else
  echo -e "${RED}Migration command failed!${NC}"
  exit 1
fi