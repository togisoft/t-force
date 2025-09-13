#!/bin/bash

# Colors for terminal output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Setting up shadcn UI for T-Force frontend...${NC}"

# Change to the frontend directory
cd frontend || {
  echo -e "${RED}Error: frontend directory not found!${NC}"
  exit 1
}

# Install shadcn and its dependencies in a single command
echo -e "${YELLOW}Installing shadcn and dependencies...${NC}"
npm install --save \
  shadcn@latest \
  @radix-ui/react-icons \
  class-variance-authority \
  clsx \
  tailwind-merge \
  lucide-react \
  @radix-ui/react-slot \
  @radix-ui/react-avatar \
  @radix-ui/react-dropdown-menu \
  @radix-ui/react-dialog \
  @radix-ui/react-label \
  @radix-ui/react-toast \
  @radix-ui/react-tooltip \
  @radix-ui/react-tabs \
  @radix-ui/react-navigation-menu \
  @radix-ui/react-select \
  @radix-ui/react-checkbox \
  @radix-ui/react-switch \
  @radix-ui/react-popover \
  @radix-ui/react-separator \
  @radix-ui/react-alert-dialog \
  @hookform/resolvers \
  zod \
  react-hook-form

# Initialize shadcn
echo -e "${YELLOW}Initializing shadcn...${NC}"
npx shadcn init --yes

# Add commonly used components in a single command
echo -e "${YELLOW}Adding commonly used components...${NC}"
npx shadcn add \
  button \
  card \
  input \
  form \
  avatar \
  dropdown-menu \
  dialog \
  label \
  toast \
  tooltip \
  tabs \
  navigation-menu \
  select \
  checkbox \
  switch \
  popover \
  separator \
  alert-dialog

echo -e "${GREEN}shadcn UI setup completed successfully!${NC}"
echo -e "You can now use shadcn UI components in your frontend."