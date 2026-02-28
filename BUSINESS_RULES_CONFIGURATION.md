# Business Rules System - Configuration Guide

This guide explains how to configure and manage the Business Rules System.

## Table of Contents

- [Overview](#overview)
- [Pricing Rules](#pricing-rules)
- [Loyalty Configuration](#loyalty-configuration)
- [Availability Management](#availability-management)
- [Prep Time Configuration](#prep-time-configuration)
- [Database Schema](#database-schema)
- [Configuration Examples](#configuration-examples)

## Overview

The Business Rules System is designed to be configured entirely through the database and API endpoints, without requiring code deployments. All configurations are cached for performance and can be updated in real-time.

### Configuration Storage

All configurations are stored in PostgreSQL tables:
- `coffee_availability` - Coffee availability status and time windows
- `pricing_rules` - Dynamic pricing rules with JSON configurations
- `prep_time_config` - Preparation time settings per coffee
- `loyalty_config` - Loyalty program settings
- `customer_loyalty` - Customer loyalty balances

### Cache Behavior

- **TTL**: 60 seconds
- **Warm-up**: Cache is pre-loaded on application startup
- **Invalidation**: Automatic refresh when TTL expires
- **Hit Rate**: Target > 90% (typically 95-98%)

## Pricing Rules

Pricing rules allow you to create dynamic discounts based on time, quantity, or promotions.

### Rule Types

#### 1. Time-Based Rules

Apply discounts during specific time windows (e.g., happy hour).

**Configuration:**

```json
{
  "rule_type": "time_based",
  "description": "Happy Hour - 20% off all drinks",
  "discount_type": "percentage",
  "discount_value": 20.0,
  "priority": 10,
  "time_ranges": [
    {
      "start": "15:00",
      "end": "17:00"
    },
    {
      "start": "20:00",
      "end": "22:00"
    }
  ]
}
```

**Fields:**
- `time_ranges`: Array of time windows in 24-hour format (HH:MM)
- Multiple time ranges can be specified for the same rule
- Time ranges are checked against the current server time

**Example Use Cases:**
- Happy hour discounts (3-5 PM)
- Late night specials (8-10 PM)
- Morning rush discounts (7-9 AM)
- Weekend specials (all day Saturday/Sunday)

#### 2. Quantity-Based Rules

Apply discounts when customers order multiple items.

**Configuration:**

```json
{
  "rule_type": "quantity_based",
  "description": "Buy 3 or more, get 15% off",
  "discount_type": "percentage",
  "discount_value": 15.0,
  "priority": 5,
  "min_quantity": 3
}
```

**Fields:**
- `min_quantity`: Minimum number of items required to trigger the discount
- Applies to total order quantity, not per-item

**Example Use Cases:**
- Bulk discounts (buy 5, get 10% off)
- Buy 2 get 1 free (33% discount with min_quantity: 3)
- Family pack deals (buy 4+, get 20% off)

#### 3. Promotional Rules

Apply discounts for special promotions or events.

**Configuration:**

```json
{
  "rule_type": "promotional",
  "description": "Valentine's Day Special - $5 off",
  "discount_type": "fixed_amount",
  "discount_value": 5.0,
  "priority": 15,
  "valid_from": "2026-02-14T00:00:00Z",
  "valid_until": "2026-02-14T23:59:59Z",
  "coffee_ids": [1, 3, 5]
}
```

**Fields:**
- `valid_from`: Start date/time (ISO 8601 format)
- `valid_until`: End date/time (ISO 8601 format)
- `coffee_ids`: Specific coffee IDs (null = all coffees)

**Example Use Cases:**
- Holiday specials (Christmas, Valentine's Day)
- New product launches (50% off for first week)
- Seasonal promotions (Summer sale)
- Flash sales (24-hour discount)

### Discount Types

#### Percentage Discount

Reduces the price by a percentage.

```json
{
  "discount_type": "percentage",
  "discount_value": 20.0
}
```

- Value: 0-100 (e.g., 20 = 20% off)
- Applied to base price: `final_price = base_price * (1 - discount_value / 100)`

#### Fixed Amount Discount

Reduces the price by a fixed dollar amount.

```json
{
  "discount_type": "fixed_amount",
  "discount_value": 5.0
}
```

- Value: Dollar amount (e.g., 5.0 = $5 off)
- Applied to base price: `final_price = base_price - discount_value`
- Final price is clamped to $0 (never negative)

### Rule Priority

Rules are applied in priority order (highest first).

**Priority Guidelines:**
- **1-5**: Low priority (general discounts)
- **6-10**: Medium priority (time-based, quantity-based)
- **11-15**: High priority (promotions, special events)
- **16-20**: Critical priority (override all other rules)

**Example:**
```
Priority 15: Valentine's Day Special (applied first)
Priority 10: Happy Hour (applied second)
Priority 5: Bulk discount (applied third)
```

### Combination Strategies

Multiple rules can be combined using different strategies:

#### 1. Additive Strategy

Sum all discounts together.

```
Base price: $10
Rule 1: 10% off = $1
Rule 2: 5% off = $0.50
Total discount: $1.50
Final price: $8.50
```

#### 2. Multiplicative Strategy

Apply discounts sequentially.

```
Base price: $10
Rule 1: 10% off = $9
Rule 2: 5% off on $9 = $8.55
Final price: $8.55
```

#### 3. Best Price Strategy (Default)

Try all combinations and choose the lowest price.

```
Base price: $10
Option 1 (additive): $8.50
Option 2 (multiplicative): $8.55
Option 3 (rule 1 only): $9.00
Option 4 (rule 2 only): $9.50
Best price: $8.50 (additive)
```

### Rule Management

#### Creating Rules

```bash
curl -X POST http://localhost:8080/api/business-rules/pricing \
  -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "rule_type": "time_based",
    "description": "Happy Hour",
    "discount_type": "percentage",
    "discount_value": 20.0,
    "priority": 10,
    "time_ranges": [{"start": "15:00", "end": "17:00"}]
  }'
```

#### Updating Rules

```bash
curl -X PUT http://localhost:8080/api/business-rules/pricing/1 \
  -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "rule_type": "time_based",
    "description": "Extended Happy Hour",
    "discount_type": "percentage",
    "discount_value": 25.0,
    "priority": 10,
    "time_ranges": [{"start": "15:00", "end": "18:00"}]
  }'
```

#### Deactivating Rules

```bash
curl -X DELETE http://localhost:8080/api/business-rules/pricing/1 \
  -H "Authorization: Bearer <admin_token>"
```

## Loyalty Configuration

Configure how customers earn loyalty points.

### Configuration Structure

```json
{
  "points_per_dollar": 1.5,
  "bonus_multipliers": {
    "1": 2.0,
    "5": 1.5,
    "10": 3.0
  }
}
```

### Fields

#### points_per_dollar

Base points earned per dollar spent.

- **Type**: Decimal
- **Range**: >= 0
- **Example**: 1.5 means $10 order = 15 base points

#### bonus_multipliers

Additional multipliers for specific coffee items.

- **Type**: Object (coffee_id -> multiplier)
- **Range**: >= 1.0
- **Example**: Coffee ID 1 with 2.0 multiplier = double points

### Points Calculation

```
Base Points = order_total * points_per_dollar
Bonus Points = sum(item_price * item_quantity * (multiplier - 1))
Total Points = Base Points + Bonus Points (rounded down)
```

**Example:**

```
Order: 2x Coffee #1 ($5 each), 1x Coffee #5 ($4)
Configuration:
  - points_per_dollar: 1.5
  - bonus_multipliers: {"1": 2.0, "5": 1.5}

Calculation:
  - Order total: $14
  - Base points: 14 * 1.5 = 21
  - Bonus for Coffee #1: 10 * (2.0 - 1.0) = 10
  - Bonus for Coffee #5: 4 * (1.5 - 1.0) = 2
  - Total: 21 + 10 + 2 = 33 points
```

### Updating Configuration

```bash
curl -X PUT http://localhost:8080/api/business-rules/loyalty-config \
  -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "points_per_dollar": 1.5,
    "bonus_multipliers": {"1": 2.0, "5": 1.5}
  }'
```

## Availability Management

Control which coffees can be ordered and when.

### Availability Statuses

#### Available

Coffee is available for ordering.

```json
{
  "coffee_id": 1,
  "status": "available"
}
```

#### Out of Stock

Coffee is temporarily unavailable.

```json
{
  "coffee_id": 2,
  "status": "out_of_stock",
  "reason": "Waiting for delivery"
}
```

#### Seasonal

Coffee is only available during specific times.

```json
{
  "coffee_id": 3,
  "status": "seasonal",
  "available_from": "08:00",
  "available_until": "14:00"
}
```

#### Discontinued

Coffee is permanently unavailable.

```json
{
  "coffee_id": 4,
  "status": "discontinued",
  "reason": "Product discontinued"
}
```

### Time-Based Availability

Restrict availability to specific time windows.

```json
{
  "coffee_id": 5,
  "status": "available",
  "available_from": "06:00",
  "available_until": "11:00"
}
```

- Times are in 24-hour format (HH:MM)
- Checked against current server time
- Useful for breakfast-only or lunch-only items

### Updating Availability

```bash
curl -X POST http://localhost:8080/api/business-rules/availability \
  -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "coffee_id": 1,
    "status": "available",
    "available_from": "08:00",
    "available_until": "20:00"
  }'
```

## Prep Time Configuration

Configure how long each coffee takes to prepare.

### Configuration Structure

```json
{
  "coffee_id": 1,
  "base_minutes": 5,
  "per_additional_item": 2
}
```

### Fields

#### base_minutes

Base preparation time for the first item.

- **Type**: Integer
- **Range**: >= 1
- **Example**: 5 means first item takes 5 minutes

#### per_additional_item

Additional time for each extra item of the same coffee.

- **Type**: Integer
- **Range**: >= 0
- **Example**: 2 means each additional item adds 2 minutes

### Calculation

```
Total Prep Time = base_minutes + (quantity - 1) * per_additional_item + queue_delay
```

**Example:**

```
Coffee #1: base_minutes=5, per_additional_item=2
Order: 3x Coffee #1
Queue: 2 orders ahead (10 minutes total)

Calculation:
  - Base time: 5 minutes
  - Additional items: (3 - 1) * 2 = 4 minutes
  - Queue delay: 10 minutes
  - Total: 5 + 4 + 10 = 19 minutes
```

### Updating Configuration

```bash
curl -X PUT http://localhost:8080/api/business-rules/prep-time/1 \
  -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "base_minutes": 5,
    "per_additional_item": 2
  }'
```

## Database Schema

### Direct Database Access

For bulk operations or migrations, you can directly modify the database tables.

#### Pricing Rules Table

```sql
-- Insert a new pricing rule
INSERT INTO pricing_rules (
  rule_type, priority, rule_config, coffee_ids, 
  is_active, valid_from, valid_until
) VALUES (
  'time_based', 10, 
  '{"time_ranges": [{"start": "15:00", "end": "17:00"}], 
    "discount_type": "percentage", "discount_value": 20.0, 
    "description": "Happy Hour"}',
  NULL, true, NOW(), NULL
);

-- Deactivate a rule
UPDATE pricing_rules SET is_active = false WHERE rule_id = '...';

-- Update rule priority
UPDATE pricing_rules SET priority = 15 WHERE rule_id = '...';
```

#### Loyalty Configuration Table

```sql
-- Update loyalty configuration
UPDATE loyalty_config SET 
  points_per_dollar = 1.5,
  bonus_multipliers = '{"1": 2.0, "5": 1.5}'::jsonb,
  updated_at = NOW()
WHERE config_id = 1;
```

#### Coffee Availability Table

```sql
-- Update availability
UPDATE coffee_availability SET 
  status = 'available',
  available_from = '08:00'::time,
  available_until = '20:00'::time,
  updated_at = NOW()
WHERE coffee_id = 1;

-- Mark as out of stock
UPDATE coffee_availability SET 
  status = 'out_of_stock',
  reason = 'Waiting for delivery',
  updated_at = NOW()
WHERE coffee_id = 2;
```

## Configuration Examples

### Example 1: Coffee Shop with Happy Hour

```bash
# Create happy hour rule (3-5 PM, 20% off)
curl -X POST http://localhost:8080/api/business-rules/pricing \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "rule_type": "time_based",
    "description": "Happy Hour - 20% off",
    "discount_type": "percentage",
    "discount_value": 20.0,
    "priority": 10,
    "time_ranges": [{"start": "15:00", "end": "17:00"}]
  }'

# Configure loyalty (1 point per dollar)
curl -X PUT http://localhost:8080/api/business-rules/loyalty-config \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "points_per_dollar": 1.0,
    "bonus_multipliers": {}
  }'
```

### Example 2: Breakfast-Only Items

```bash
# Set espresso as breakfast-only (6-11 AM)
curl -X POST http://localhost:8080/api/business-rules/availability \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "coffee_id": 1,
    "status": "available",
    "available_from": "06:00",
    "available_until": "11:00"
  }'

# Configure quick prep time for espresso
curl -X PUT http://localhost:8080/api/business-rules/prep-time/1 \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "base_minutes": 3,
    "per_additional_item": 1
  }'
```

### Example 3: Seasonal Promotion

```bash
# Create summer promotion (June-August, $3 off iced drinks)
curl -X POST http://localhost:8080/api/business-rules/pricing \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "rule_type": "promotional",
    "description": "Summer Special - $3 off iced drinks",
    "discount_type": "fixed_amount",
    "discount_value": 3.0,
    "priority": 12,
    "valid_from": "2026-06-01T00:00:00Z",
    "valid_until": "2026-08-31T23:59:59Z",
    "coffee_ids": [3, 5, 7]
  }'
```

## Best Practices

1. **Test rules in staging first** - Verify pricing calculations before production
2. **Use descriptive names** - Make rule descriptions clear and specific
3. **Set appropriate priorities** - Higher priority for time-sensitive promotions
4. **Monitor metrics** - Check `/api/business-rules/metrics` regularly
5. **Review audit logs** - Verify rules are being applied correctly
6. **Cache warming** - Ensure cache is warmed on startup for best performance
7. **Backup configurations** - Export rules before making bulk changes
8. **Document custom rules** - Keep a record of special promotions and their dates

## Troubleshooting

### Rules Not Applying

1. Check if rule is active: `is_active = true`
2. Verify date range: `valid_from` and `valid_until`
3. Check priority: Higher priority rules may override
4. Review cache: Wait 60 seconds for cache refresh or restart server

### Slow Performance

1. Check metrics: `GET /api/business-rules/metrics`
2. Look for slow operations (> 100ms)
3. Verify cache hit rate (should be > 90%)
4. Check database indexes are in place

### Incorrect Pricing

1. Review applied rules in audit log
2. Check combination strategy (additive vs multiplicative)
3. Verify discount calculations manually
4. Test with single rule to isolate issue
