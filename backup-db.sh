#!/bin/bash
# Backup PostgreSQL database before migrations
# Uses custom format (best practice) - compressed, supports parallel restore

if [ -z "$1" ]; then
    echo "Usage: ./backup-db.sh <environment> [output_file]"
    echo "Example: ./backup-db.sh dev"
    echo "Example: ./backup-db.sh prod custom_backup.dump"
    echo ""
    echo "Available environments: dev, prod"
    exit 1
fi

ENV_NAME=$1
ENV_FILE=".env.${ENV_NAME}"

if [ ! -f "$ENV_FILE" ]; then
    echo "✗ Error: Environment file '$ENV_FILE' not found"
    echo "  Please create $ENV_FILE with your DATABASE_URL"
    exit 1
fi

# Source the env file to get DATABASE_URL
source "$ENV_FILE"

if [ -z "$DATABASE_URL" ]; then
    echo "✗ Error: DATABASE_URL not found in $ENV_FILE"
    exit 1
fi

TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Extract database name from connection string for default filename
DB_NAME=$(echo "$DATABASE_URL" | sed -n 's|.*/\([^?]*\).*|\1|p')

if [ -z "$2" ]; then
    BACKUP_FILE="${DB_NAME}_${ENV_NAME}_${TIMESTAMP}.dump"
else
    BACKUP_FILE=$2
fi

echo "Backing up $ENV_NAME environment to $BACKUP_FILE..."
pg_dump -Fc -f "$BACKUP_FILE" "$DATABASE_URL"

if [ $? -eq 0 ]; then
    echo "✓ Backup complete: $BACKUP_FILE"
    echo "  Restore with: pg_restore -d <connection_string> $BACKUP_FILE"
else
    echo "✗ Backup failed"
    exit 1
fi
