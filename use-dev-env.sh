#!/bin/bash
# Switch to development environment
cp .env.dev .env
echo "✓ Switched to development environment"
echo "  DATABASE_URL=$(grep DATABASE_URL .env | cut -d= -f2)"
