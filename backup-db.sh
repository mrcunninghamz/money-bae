#!/bin/bash
# Backup PostgreSQL database before migrations
# Uses custom format (best practice) - compressed, supports parallel restore

if [ -z "$1" ]; then
    echo "Usage: ./backup-db.sh <database_name>"
    echo "Example: ./backup-db.sh money_bae"
    exit 1
fi

DB_NAME=$1
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${DB_NAME}_${TIMESTAMP}.dump"

echo "Backing up $DB_NAME to $BACKUP_FILE..."
pg_dump -Fc -f "$BACKUP_FILE" "$DB_NAME"

if [ $? -eq 0 ]; then
    echo "✓ Backup complete: $BACKUP_FILE"
    echo "  Restore with: pg_restore -d $DB_NAME $BACKUP_FILE"
else
    echo "✗ Backup failed"
    exit 1
fi
