#!/bin/bash

# Script to seed production database with sample coffee data
# This script should be run after the database is up and migrations are complete

set -e

echo "🌱 Seeding production database with sample coffee data..."

# Check if docker-compose is available
if ! command -v docker-compose &> /dev/null; then
    echo "❌ Error: docker-compose is not installed"
    exit 1
fi

# Check if database container is running
if ! docker-compose ps db | grep -q "Up"; then
    echo "⚠️  Database container is not running. Starting it now..."
    docker-compose up -d db
    echo "⏳ Waiting for database to be ready..."
    sleep 5
fi

# Run the seed script
echo "📝 Executing seed_data.sql..."
docker-compose exec -T db psql -U coffee_user -d coffee_db < seed_data.sql

echo "✅ Production database seeded successfully!"
echo "🔍 You can verify the data by running:"
echo "   docker-compose exec db psql -U coffee_user -d coffee_db -c 'SELECT COUNT(*) FROM coffees;'"
