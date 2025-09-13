#!/bin/bash

# T-Force Production Backup Script
# This script creates automated backups of the database and application files

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BACKUP_DIR="/backups"
DATE=$(date +%Y%m%d_%H%M%S)
DB_NAME="${POSTGRES_DB:-tforce}"
DB_USER="${POSTGRES_USER:-postgres}"
DB_HOST="db"
RETENTION_DAYS=30

echo -e "${BLUE}🚀 Starting T-Force Production Backup${NC}"
echo -e "${YELLOW}Date:${NC} $(date)"
echo -e "${YELLOW}Backup Directory:${NC} $BACKUP_DIR"
echo -e "${YELLOW}Database:${NC} $DB_NAME"

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# Database backup
echo -e "${BLUE}📊 Creating database backup...${NC}"
DB_BACKUP_FILE="$BACKUP_DIR/tforce_db_$DATE.sql.gz"
PGPASSWORD="$POSTGRES_PASSWORD" pg_dump -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" \
    --verbose --clean --no-owner --no-privileges \
    | gzip > "$DB_BACKUP_FILE"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Database backup created:${NC} $(basename "$DB_BACKUP_FILE")"
    echo -e "${YELLOW}Size:${NC} $(du -h "$DB_BACKUP_FILE" | cut -f1)"
else
    echo -e "${RED}❌ Database backup failed!${NC}"
    exit 1
fi

# Create backup manifest
MANIFEST_FILE="$BACKUP_DIR/backup_manifest_$DATE.json"
cat > "$MANIFEST_FILE" << EOF
{
  "backup_date": "$(date -Iseconds)",
  "backup_type": "full",
  "database": {
    "name": "$DB_NAME",
    "backup_file": "$(basename "$DB_BACKUP_FILE")",
    "size": "$(du -h "$DB_BACKUP_FILE" | cut -f1)"
  },
  "system_info": {
    "hostname": "$(hostname)",
    "docker_version": "$(docker --version 2>/dev/null || echo 'N/A')"
  }
}
EOF

echo -e "${GREEN}✅ Backup manifest created:${NC} $(basename "$MANIFEST_FILE")"

# Clean up old backups (keep last 30 days)
echo -e "${BLUE}🧹 Cleaning up old backups...${NC}"
find "$BACKUP_DIR" -name "tforce_db_*.sql.gz" -mtime +$RETENTION_DAYS -delete
find "$BACKUP_DIR" -name "backup_manifest_*.json" -mtime +$RETENTION_DAYS -delete

echo -e "${GREEN}✅ Cleanup completed${NC}"

# Show backup summary
echo -e "${BLUE}📋 Backup Summary:${NC}"
echo -e "${YELLOW}Total backups:${NC} $(find "$BACKUP_DIR" -name "tforce_db_*.sql.gz" | wc -l)"
echo -e "${YELLOW}Total size:${NC} $(du -sh "$BACKUP_DIR" | cut -f1)"
echo -e "${YELLOW}Retention:${NC} $RETENTION_DAYS days"

echo -e "${GREEN}🎉 T-Force backup completed successfully!${NC}" 