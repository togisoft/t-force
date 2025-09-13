#!/bin/bash

# Colors for terminal output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Running tests for T-Force...${NC}"

# Check if .env file exists
if [ ! -f .env ]; then
  echo -e "${RED}Error: .env file not found!${NC}"
  echo -e "Please create a .env file based on .env.template"
  echo -e "See OAUTH_SETUP.md for detailed instructions"
  exit 1
fi

# Test 1: Check if Docker is running
echo -e "\n${YELLOW}Test 1: Checking if Docker is running...${NC}"
if ! docker info > /dev/null 2>&1; then
  echo -e "${RED}Error: Docker is not running!${NC}"
  echo -e "Please start Docker and try again"
  exit 1
else
  echo -e "${GREEN}Docker is running!${NC}"
fi

# Test 2: Check if required environment variables are set
echo -e "\n${YELLOW}Test 2: Checking environment variables...${NC}"
required_vars=("DATABASE_URL" "NEXTAUTH_SECRET" "GOOGLE_CLIENT_ID" "GOOGLE_CLIENT_SECRET" "GITHUB_CLIENT_ID" "GITHUB_CLIENT_SECRET" "NEXT_PUBLIC_API_URL")
missing_vars=()

for var in "${required_vars[@]}"; do
  if ! grep -q "^$var=" .env || grep -q "^$var=REPLACE_WITH" .env; then
    missing_vars+=("$var")
  fi
done

if [ ${#missing_vars[@]} -ne 0 ]; then
  echo -e "${RED}Error: The following environment variables are missing or not properly set:${NC}"
  for var in "${missing_vars[@]}"; do
    echo -e "  - $var"
  done
  echo -e "Please update your .env file with the correct values"
  echo -e "See OAUTH_SETUP.md for detailed instructions"
  exit 1
else
  echo -e "${GREEN}All required environment variables are set!${NC}"
fi

# Test 3: Check if frontend components exist
echo -e "\n${YELLOW}Test 3: Checking frontend components...${NC}"
required_components=("button.tsx" "card.tsx" "separator.tsx" "avatar.tsx" "tabs.tsx")
missing_components=()

for component in "${required_components[@]}"; do
  if [ ! -f "frontend/components/ui/$component" ]; then
    missing_components+=("$component")
  fi
done

if [ ${#missing_components[@]} -ne 0 ]; then
  echo -e "${RED}Error: The following UI components are missing:${NC}"
  for component in "${missing_components[@]}"; do
    echo -e "  - $component"
  done
  echo -e "Please run the setup-shadcn.sh script to install the missing components"
  exit 1
else
  echo -e "${GREEN}All required UI components are present!${NC}"
fi

# Test 4: Check if utils.ts exists
echo -e "\n${YELLOW}Test 4: Checking utils.ts...${NC}"
if [ ! -f "frontend/lib/utils.ts" ]; then
  echo -e "${RED}Error: utils.ts is missing!${NC}"
  echo -e "Please create the utils.ts file in the frontend/lib directory"
  exit 1
else
  echo -e "${GREEN}utils.ts is present!${NC}"
fi

# Test 5: Run database migrations
echo -e "\n${YELLOW}Test 5: Running database migrations...${NC}"
echo -e "Starting database container..."
docker compose -f docker-compose.dev.yml up -d db

# Wait for database to be ready
echo -e "Waiting for database to be ready..."
sleep 5

# Run migrations
echo -e "Running migrations..."
cd backend
if ! sea-orm-cli migrate up; then
  echo -e "${RED}Error: Failed to run migrations!${NC}"
  echo -e "Please check your database configuration and try again"
  exit 1
else
  echo -e "${GREEN}Migrations completed successfully!${NC}"
fi
cd ..

# Test 6: Build backend
echo -e "\n${YELLOW}Test 6: Building backend...${NC}"
cd backend
if ! cargo build; then
  echo -e "${RED}Error: Failed to build backend!${NC}"
  echo -e "Please check your Rust code and try again"
  exit 1
else
  echo -e "${GREEN}Backend built successfully!${NC}"
fi
cd ..

# Test 7: Build frontend
echo -e "\n${YELLOW}Test 7: Building frontend...${NC}"
cd frontend
if ! npm run build; then
  echo -e "${RED}Error: Failed to build frontend!${NC}"
  echo -e "Please check your frontend code and try again"
  exit 1
else
  echo -e "${GREEN}Frontend built successfully!${NC}"
fi
cd ..

# All tests passed
echo -e "\n${GREEN}All tests passed!${NC}"
echo -e "Your T-Force application is ready to run"
echo -e "Run ${YELLOW}./scripts/docker.sh dev${NC} to start the application in development mode"
echo -e "Open ${YELLOW}http://localhost:3000${NC} in your browser to access the application"