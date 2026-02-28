# Business Rules System - Deployment Guide

This guide covers deploying the Business Rules System to production.

## Pre-Deployment Checklist

### Database

- [ ] All migrations have been run successfully
- [ ] Database indexes are in place
- [ ] Seed data has been loaded (if applicable)
- [ ] Database connection pool is configured appropriately
- [ ] Backup strategy is in place

### Configuration

- [ ] Environment variables are set correctly
- [ ] JWT_SECRET is configured
- [ ] DATABASE_URL points to production database
- [ ] Cache TTL is appropriate for your use case (default: 60 seconds)

### Testing

- [ ] All unit tests pass
- [ ] Integration tests pass (if implemented)
- [ ] Performance tests meet < 100ms requirement (if implemented)
- [ ] Manual testing of all endpoints completed

### Security

- [ ] Admin endpoints require authentication
- [ ] JWT tokens are properly validated
- [ ] SQL injection protection is in place (sqlx handles this)
- [ ] Rate limiting is configured (recommended for production)

## Deployment Steps

### 1. Database Migration

Run all migrations in order:

```bash
# Using Docker
docker-compose exec backend sqlx migrate run

# Or directly with sqlx-cli
sqlx migrate run --database-url $DATABASE_URL
```

Verify migrations:

```bash
# Check migration history
docker-compose exec backend sqlx migrate info
```

### 2. Seed Initial Data

Load seed data for business rules:

```bash
# Using Docker
docker-compose exec -T backend psql $DATABASE_URL < seed_data.sql

# Or using the seed script
./scripts/seed_production.sh
```

Verify seed data:

```sql
-- Check loyalty config
SELECT * FROM loyalty_config;

-- Check prep time config
SELECT * FROM prep_time_config;

-- Check coffee availability
SELECT * FROM coffee_availability;
```

### 3. Build and Deploy Application

```bash
# Build the application
docker-compose build backend

# Start the application
docker-compose up -d backend

# Check logs
docker-compose logs -f backend
```

Look for these startup messages:

```
INFO Coffee API - Starting...
INFO Connecting to database...
INFO Running database migrations...
INFO Migrations completed successfully
INFO Initializing authentication service...
INFO Initializing business rules engine...
INFO Warming business rules cache...
INFO Business rules cache warmed successfully
INFO Business rules engine initialized
INFO Starting server on 0.0.0.0:8080
INFO Coffee API is running on http://0.0.0.0:8080
```

### 4. Verify Deployment

#### Health Check

```bash
# Check if server is responding
curl http://localhost:8080/api/coffees

# Check business rules metrics
curl http://localhost:8080/api/business-rules/metrics
```

#### Test Business Rules

```bash
# Test availability endpoint
curl http://localhost:8080/api/business-rules/availability/1

# Test pricing rules endpoint
curl http://localhost:8080/api/business-rules/pricing

# Test loyalty config endpoint
curl http://localhost:8080/api/business-rules/loyalty-config
```

#### Create Test Order

```bash
# Login as test user
TOKEN=$(curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password"}' \
  | jq -r '.access_token')

# Create test order
curl -X POST http://localhost:8080/api/orders \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "items": [
      {"coffee_item_id": 1, "quantity": 2}
    ]
  }'
```

Verify:
- Order is created successfully
- Availability is validated
- Pricing rules are applied
- Prep time is estimated

### 5. Monitor Performance

#### Check Metrics

```bash
curl http://localhost:8080/api/business-rules/metrics | jq
```

Expected metrics:
- Cache hit rate: > 90%
- Average operation times: < 50ms
- Slow operations: 0 (or very few)

#### Check Logs

```bash
# Watch for slow operations
docker-compose logs backend | grep "Slow"

# Watch for errors
docker-compose logs backend | grep "ERROR"

# Watch for warnings
docker-compose logs backend | grep "WARN"
```

## Environment Variables

### Required

```bash
# Database connection
DATABASE_URL=postgresql://user:password@host:5432/database

# JWT authentication
JWT_SECRET=your-secret-key-here

# Server configuration
HOST=0.0.0.0
PORT=8080
```

### Optional

```bash
# Logging level
RUST_LOG=info

# Database pool size (default: 5)
DATABASE_MAX_CONNECTIONS=10

# Cache TTL (default: 60 seconds)
# Note: Currently hardcoded, would need code change to make configurable
```

## Production Configuration

### Database Connection Pool

Adjust pool size based on expected load:

```rust
// In db.rs or main.rs
let pool = PgPoolOptions::new()
    .max_connections(10)  // Adjust based on load
    .connect(&database_url)
    .await?;
```

### Cache TTL

Default is 60 seconds. Adjust in `config_store.rs` if needed:

```rust
const CACHE_TTL: Duration = Duration::from_secs(60);
```

Considerations:
- Lower TTL (30s): More database queries, fresher data
- Higher TTL (120s): Fewer database queries, staler data

### Performance Tuning

#### Database Indexes

Verify indexes are in place:

```sql
-- Check indexes
SELECT tablename, indexname, indexdef 
FROM pg_indexes 
WHERE schemaname = 'public' 
  AND tablename IN ('pricing_rules', 'rule_audit_log');
```

#### Query Performance

Monitor slow queries:

```sql
-- Enable slow query logging
ALTER DATABASE your_database SET log_min_duration_statement = 100;

-- Check slow queries
SELECT query, calls, total_time, mean_time
FROM pg_stat_statements
WHERE mean_time > 100
ORDER BY mean_time DESC;
```

## Monitoring

### Application Metrics

Monitor these endpoints:

```bash
# Performance metrics
curl http://localhost:8080/api/business-rules/metrics

# Health check
curl http://localhost:8080/api/coffees
```

### Database Metrics

Monitor these queries:

```sql
-- Active connections
SELECT count(*) FROM pg_stat_activity;

-- Cache hit rate
SELECT 
  sum(heap_blks_hit) / (sum(heap_blks_hit) + sum(heap_blks_read)) as cache_hit_ratio
FROM pg_statio_user_tables;

-- Table sizes
SELECT 
  schemaname, tablename,
  pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```

### Log Monitoring

Set up log aggregation for:
- Slow operations (> 100ms)
- Database errors
- Authentication failures
- Cache misses (if rate is low)

## Backup and Recovery

### Database Backup

```bash
# Backup database
pg_dump $DATABASE_URL > backup_$(date +%Y%m%d_%H%M%S).sql

# Backup specific tables
pg_dump $DATABASE_URL \
  -t pricing_rules \
  -t loyalty_config \
  -t coffee_availability \
  -t prep_time_config \
  > business_rules_backup_$(date +%Y%m%d_%H%M%S).sql
```

### Configuration Export

```bash
# Export pricing rules
curl http://localhost:8080/api/business-rules/pricing > pricing_rules.json

# Export loyalty config
curl http://localhost:8080/api/business-rules/loyalty-config > loyalty_config.json
```

### Recovery

```bash
# Restore database
psql $DATABASE_URL < backup.sql

# Or restore specific tables
psql $DATABASE_URL < business_rules_backup.sql
```

## Scaling Considerations

### Horizontal Scaling

The Business Rules System is designed for horizontal scaling:

- **Stateless**: No server-side state (except cache)
- **Cache**: Each instance has its own cache (60s TTL ensures consistency)
- **Database**: Shared PostgreSQL database

To scale horizontally:

1. Deploy multiple instances behind a load balancer
2. Ensure all instances connect to the same database
3. Monitor cache hit rates across instances
4. Consider Redis for shared cache (future enhancement)

### Vertical Scaling

To handle more load on a single instance:

1. Increase database connection pool size
2. Increase server resources (CPU, RAM)
3. Optimize database queries (add indexes)
4. Increase cache TTL (if acceptable)

## Troubleshooting

### Cache Not Warming

**Symptom:** Startup logs show cache warming failure

**Solution:**
```bash
# Check database connectivity
docker-compose exec backend psql $DATABASE_URL -c "SELECT 1"

# Check if tables exist
docker-compose exec backend psql $DATABASE_URL -c "\dt"

# Check if seed data is loaded
docker-compose exec backend psql $DATABASE_URL -c "SELECT * FROM loyalty_config"
```

### Slow Performance

**Symptom:** Operations taking > 100ms

**Solution:**
```bash
# Check metrics
curl http://localhost:8080/api/business-rules/metrics

# Check database performance
docker-compose exec backend psql $DATABASE_URL -c "
  SELECT query, calls, mean_time 
  FROM pg_stat_statements 
  WHERE mean_time > 100 
  ORDER BY mean_time DESC
"

# Check cache hit rate
# Should be > 90%
```

### Rules Not Applying

**Symptom:** Pricing rules or availability checks not working

**Solution:**
```bash
# Check if rules are active
docker-compose exec backend psql $DATABASE_URL -c "
  SELECT rule_id, rule_type, is_active, valid_from, valid_until 
  FROM pricing_rules
"

# Check audit log
docker-compose exec backend psql $DATABASE_URL -c "
  SELECT * FROM rule_audit_log 
  ORDER BY created_at DESC 
  LIMIT 10
"

# Wait for cache refresh (60 seconds) or restart
docker-compose restart backend
```

## Rollback Procedure

If deployment fails:

1. **Stop the application:**
   ```bash
   docker-compose stop backend
   ```

2. **Restore database backup:**
   ```bash
   psql $DATABASE_URL < backup.sql
   ```

3. **Revert to previous version:**
   ```bash
   git checkout <previous-tag>
   docker-compose build backend
   docker-compose up -d backend
   ```

4. **Verify rollback:**
   ```bash
   curl http://localhost:8080/api/business-rules/metrics
   ```

## Post-Deployment

### Verify System Health

- [ ] All endpoints responding correctly
- [ ] Cache hit rate > 90%
- [ ] Average operation times < 50ms
- [ ] No slow operations in logs
- [ ] Test orders complete successfully
- [ ] Loyalty points awarded correctly

### Monitor for 24 Hours

- [ ] Check metrics every 4 hours
- [ ] Review logs for errors
- [ ] Monitor database performance
- [ ] Verify cache effectiveness
- [ ] Check audit log for anomalies

### Documentation

- [ ] Update deployment notes
- [ ] Document any configuration changes
- [ ] Record any issues encountered
- [ ] Update runbook with lessons learned

## Support

For production issues:

1. Check logs: `docker-compose logs backend`
2. Check metrics: `GET /api/business-rules/metrics`
3. Check audit log: Query `rule_audit_log` table
4. Review this deployment guide
5. Consult API documentation: `BUSINESS_RULES_API.md`
6. Review configuration guide: `BUSINESS_RULES_CONFIGURATION.md`
