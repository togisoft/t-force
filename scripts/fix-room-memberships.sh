#!/bin/bash

# Fix Room Memberships Script
# This script adds room creators as members for existing rooms

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔧 Fixing Room Memberships${NC}"
echo -e "${YELLOW}Date:${NC} $(date)"
echo ""

# Check if database container is running
if ! docker ps | grep -q "tforce-db"; then
    echo -e "${RED}❌ Database container is not running!${NC}"
    echo -e "${YELLOW}Please start the development environment first:${NC}"
    echo -e "  ./scripts/dev.sh"
    exit 1
fi

echo -e "${BLUE}📊 Checking existing room memberships...${NC}"

# Get database name
DB_NAME=$(docker exec tforce-db psql -U postgres -t -c "SELECT datname FROM pg_database WHERE datname LIKE '%authforce%' OR datname LIKE '%tforce%';" | head -1 | xargs)

if [ -z "$DB_NAME" ]; then
    echo -e "${RED}❌ Could not find database!${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Found database: $DB_NAME${NC}"

# Check rooms without creator memberships
echo -e "${BLUE}🔍 Checking rooms without creator memberships...${NC}"

ROOMS_TO_FIX=$(docker exec tforce-db psql -U postgres -d "$DB_NAME" -t -c "
SELECT r.id, r.name, r.created_by 
FROM chat_rooms r 
LEFT JOIN room_memberships rm ON r.id = rm.room_id AND r.created_by = rm.user_id 
WHERE rm.id IS NULL;
")

if [ -z "$ROOMS_TO_FIX" ]; then
    echo -e "${GREEN}✅ All rooms already have creator memberships!${NC}"
    exit 0
fi

echo -e "${YELLOW}Found rooms to fix:${NC}"
echo "$ROOMS_TO_FIX"
echo ""

# Add creator memberships
echo -e "${BLUE}🔧 Adding creator memberships...${NC}"

docker exec tforce-db psql -U postgres -d "$DB_NAME" -c "
INSERT INTO room_memberships (id, user_id, room_id, joined_at)
SELECT 
    gen_random_uuid() as id,
    r.created_by as user_id,
    r.id as room_id,
    r.created_at as joined_at
FROM chat_rooms r 
LEFT JOIN room_memberships rm ON r.id = rm.room_id AND r.created_by = rm.user_id 
WHERE rm.id IS NULL;
"

echo -e "${GREEN}✅ Creator memberships added successfully!${NC}"

# Show updated member counts
echo -e "${BLUE}📊 Updated member counts:${NC}"
docker exec tforce-db psql -U postgres -d "$DB_NAME" -c "
SELECT 
    r.name,
    r.room_code,
    COUNT(rm.id) as member_count,
    r.created_by
FROM chat_rooms r 
LEFT JOIN room_memberships rm ON r.id = rm.room_id 
GROUP BY r.id, r.name, r.room_code, r.created_by 
ORDER BY r.created_at DESC;
"

echo -e "${GREEN}🎉 Room memberships fixed successfully!${NC}" 