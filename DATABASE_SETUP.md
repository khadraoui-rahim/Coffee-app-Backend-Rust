# Database Setup Guide

This document explains how the production and test databases are separated to ensure test data doesn't affect production data.

## Overview

The Coffee Menu Backend uses **two separate PostgreSQL databases**:

1. **Production Database** (`coffee_db`) - Port 5432
   - Used by the API server
   - Contains real coffee menu data
   - Data persists across restarts

2. **Test Database** (`coffee_test_db`) - Port 5433
   - Used exclusively for running tests
   - Data is truncated before each test
   - Completely isolated from production

## Why Separate Databases?

Previously, tests were using the same database as production, which caused:
- ❌ Test data appearing in production
- ❌ Tests deleting production data (TRUNCATE operations)
- ❌ Data inconsistencies
- ❌ Inability to run tests safely in production environments

With separate databases:
- ✅ Tests can safely truncate tables without affecting production
- ✅ Production data remains intact during testing
- ✅ Tests are isolated and repeatable
- ✅ Safe to run tests at any time

## Docker Compose Configuration

The `docker-compose.yml` file defines both databases:

```yaml
services:
  # Production database
  db:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: coffee_db
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  # Test database
  test_db:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: coffee_test_db
    ports:
      - "5433:5432"
    volumes:
      - test_postgres_data:/var/lib/postgresql/data

  # API server (uses production db)
  api:
    environment:
      DATABASE_URL: postgresql://coffee_user:coffee_pass@db:5432/coffee_db

  # Test runner (uses test db)
  test:
    environment:
      DATABASE_URL: postgresql://coffee_user:coffee_pass@test_db:5432/coffee_test_db
```

## Starting the Databases

### Start Production Database Only
```bash
docker-compose up -d db
```

### Start Both Databases
```bash
docker-compose up -d db test_db
```

### Start Everything (API + Databases)
```bash
docker-compose up -d
```

## Seeding Production Data

After starting the production database, seed it with sample coffee items:

### Using the Seed Script (Recommended)

**Linux/Mac:**
```bash
chmod +x scripts/seed_production.sh
./scripts/seed_production.sh
```

**Windows:**
```cmd
scripts\seed_production.bat
```

### Manual Seeding

```bash
# Make sure the database is running
docker-compose up -d db

# Run the seed script
docker-compose exec -T db psql -U coffee_user -d coffee_db < seed_data.sql
```

### Verify Seeded Data

```bash
docker-compose exec db psql -U coffee_user -d coffee_db -c "SELECT COUNT(*) FROM coffees;"
```

You should see 12 coffee items.

## Running Tests

Tests automatically use the test database:

```bash
# Run all tests (uses test_db)
docker-compose run --rm test

# Or with cargo (if DATABASE_URL points to test_db)
cargo test
```

**Important:** Tests will:
- Connect to `coffee_test_db` (not `coffee_db`)
- Run migrations on the test database
- Truncate tables before each test
- Never touch production data

## Connecting to Databases

### Production Database

```bash
# Using docker-compose
docker-compose exec db psql -U coffee_user -d coffee_db

# Using psql directly
psql -h localhost -p 5432 -U coffee_user -d coffee_db
```

### Test Database

```bash
# Using docker-compose
docker-compose exec test_db psql -U coffee_user -d coffee_test_db

# Using psql directly
psql -h localhost -p 5433 -U coffee_user -d coffee_test_db
```

## Environment Variables

### Production (.env)
```bash
DATABASE_URL=postgresql://coffee_user:coffee_pass@localhost:5432/coffee_db
```

### Testing
The test runner automatically sets:
```bash
DATABASE_URL=postgresql://coffee_user:coffee_pass@test_db:5432/coffee_test_db
```

## Database Volumes

Data is persisted in Docker volumes:

- `postgres_data` - Production database data
- `test_postgres_data` - Test database data

### Clearing Test Data
```bash
# Remove test database volume (safe - doesn't affect production)
docker-compose down
docker volume rm coffee_app-backend_test_postgres_data
```

### Clearing Production Data (⚠️ Use with caution!)
```bash
# This will delete all production data!
docker-compose down
docker volume rm coffee_app-backend_postgres_data
```

## Migrations

Migrations run automatically on both databases:

- **Production**: Migrations run when the API starts
- **Test**: Migrations run before each test suite

To run migrations manually:

```bash
# Production database
docker-compose exec api sqlx migrate run

# Test database
docker-compose run --rm test sqlx migrate run
```

## Troubleshooting

### Tests are using production database

Check the `DATABASE_URL` environment variable in the test service:
```bash
docker-compose config | grep -A 5 "test:"
```

It should point to `test_db:5432/coffee_test_db`.

### Production data disappeared after running tests

This means tests were using the production database. Follow these steps:

1. Stop all services: `docker-compose down`
2. Verify `docker-compose.yml` has separate `db` and `test_db` services
3. Verify test service uses `test_db` in DATABASE_URL
4. Restart and reseed: 
   ```bash
   docker-compose up -d db
   ./scripts/seed_production.sh
   ```

### Port conflicts

If port 5432 or 5433 is already in use:

1. Edit `docker-compose.yml`
2. Change the port mapping (e.g., `"5434:5432"`)
3. Update your DATABASE_URL accordingly

## Best Practices

1. **Always use docker-compose for tests**: `docker-compose run --rm test`
2. **Never manually connect to test_db for production data**
3. **Seed production database after setup**: Run the seed script
4. **Back up production data regularly**: Use `pg_dump`
5. **Keep test database clean**: It's automatically cleaned before each test

## Summary

| Database | Port | Purpose | Data Persistence | Used By |
|----------|------|---------|------------------|---------|
| `coffee_db` | 5432 | Production | Persistent | API server |
| `coffee_test_db` | 5433 | Testing | Temporary | Test suite |

This separation ensures your production data is always safe, even when running tests! 🎉
