@echo off
REM Script to seed production database with sample coffee data (Windows)
REM This script should be run after the database is up and migrations are complete

echo 🌱 Seeding production database with sample coffee data...

REM Check if docker-compose is available
where docker-compose >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Error: docker-compose is not installed
    exit /b 1
)

REM Check if database container is running
docker-compose ps db | findstr "Up" >nul
if %ERRORLEVEL% NEQ 0 (
    echo ⚠️  Database container is not running. Starting it now...
    docker-compose up -d db
    echo ⏳ Waiting for database to be ready...
    timeout /t 5 /nobreak >nul
)

REM Run the seed script
echo 📝 Executing seed_data.sql...
type seed_data.sql | docker-compose exec -T db psql -U coffee_user -d coffee_db

echo ✅ Production database seeded successfully!
echo 🔍 You can verify the data by running:
echo    docker-compose exec db psql -U coffee_user -d coffee_db -c "SELECT COUNT(*) FROM coffees;"
