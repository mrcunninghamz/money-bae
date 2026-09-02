#!/bin/bash
# Switch to production environment
cp .env.prod .env
echo "✓ Switched to production environment"
echo "  DATABASE_URL=$(grep DATABASE_URL .env | cut -d= -f2)"
