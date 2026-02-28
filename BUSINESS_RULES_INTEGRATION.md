# Business Rules System Integration

This document describes the integration of the Business Rules System with the Orders API.

## Overview

The Business Rules System has been integrated into the order creation and completion flow, providing:
- Availability validation before order creation
- Dynamic pricing with configurable rules
- Preparation time estimation
- Loyalty points calculation and awarding

## Integration Points

### 1. Order Creation (Task 11.5, 11.6, 11.7)

When a customer creates an order via `POST /api/orders`, the system now:

1. **Validates Availability** - Checks if all coffee items are available
   - Queries the `coffee_availability` table
   - Checks time-based availability (available_from, available_until)
   - Returns validation errors if any items are unavailable

2. **Calculates Dynamic Pricing** - Applies pricing rules to the order
   - Loads active pricing rules from `pricing_rules` table
   - Applies time-based, quantity-based, and promotional rules
   - Uses the BestPrice combination strategy by default
   - Stores both base_price and final_price (TODO: requires migration)

3. **Estimates Preparation Time** - Calculates expected prep time
   - Loads prep time configuration from `prep_time_config` table
   - Considers base time per coffee and additional item time
   - Accounts for current queue length
   - Stores estimated_prep_minutes (TODO: requires migration)

### 2. Order Completion (Task 11.8)

When an order status is updated to "Completed" via `PATCH /api/orders/:id/status`, the system:

1. **Awards Loyalty Points** - Calculates and awards points to the customer
   - Loads loyalty configuration from `loyalty_config` table
   - Calculates base points: order_total * points_per_dollar
   - Applies bonus multipliers for specific coffee items
   - Updates customer_loyalty table with new balance
   - Stores loyalty_points_awarded (TODO: requires migration)

2. **Logs Audit Trail** - Records the loyalty award in `rule_audit_log`
   - Tracks points awarded, customer balance, and calculation breakdown
   - Provides audit trail for loyalty program management

## API Endpoints (Task 11.1-11.4)

The following management endpoints have been created (handlers implemented, business logic TODO):

### Availability Management (Admin Only)
- `POST /api/business-rules/availability` - Update coffee availability
- `GET /api/business-rules/availability/:coffee_id` - Get availability status

### Pricing Rule Management (Admin Only)
- `POST /api/business-rules/pricing` - Create pricing rule
- `PUT /api/business-rules/pricing/:rule_id` - Update pricing rule
- `DELETE /api/business-rules/pricing/:rule_id` - Deactivate pricing rule
- `GET /api/business-rules/pricing` - List active pricing rules (public)

### Configuration Management (Admin Only)
- `PUT /api/business-rules/loyalty-config` - Update loyalty configuration
- `GET /api/business-rules/loyalty-config` - Get loyalty configuration (public)
- `PUT /api/business-rules/prep-time/:coffee_id` - Update prep time config

## Architecture Changes

### AppState
Added `business_rules_engine: Arc<BusinessRulesEngine>` to the application state, making it available to all handlers.

### OrderService
- Added optional `business_rules_engine` field
- New constructor: `with_business_rules()` for integration
- Modified `create_order()` to validate, price, and estimate prep time
- Modified `update_order_status()` to award loyalty points on completion

### Error Handling
- Implemented `IntoResponse` for `BusinessRulesError`
- Added conversion from `validator::ValidationErrors`
- Proper HTTP status codes for different error types

## Database Schema Requirements

The following columns need to be added to the `orders` table (migration pending):

```sql
ALTER TABLE orders ADD COLUMN base_price DECIMAL(10, 2);
ALTER TABLE orders ADD COLUMN final_price DECIMAL(10, 2);
ALTER TABLE orders ADD COLUMN estimated_prep_minutes INTEGER;
ALTER TABLE orders ADD COLUMN loyalty_points_awarded INTEGER;
```

## Testing

Integration tests should cover:
- Order creation with unavailable items (should fail)
- Order creation with pricing rules applied
- Order creation with prep time estimation
- Order completion with loyalty points awarded
- Error scenarios (invalid rules, missing configuration)

## Next Steps

1. Implement the TODO handlers in `business_rules/handlers.rs`
2. Create database migration for orders table columns
3. Write integration tests (Task 11.9, 11.10)
4. Add API documentation with examples (Task 13.2)
5. Performance testing (Task 12)

## Notes

- Audit logging errors do not block order operations
- Loyalty points are only awarded when order status transitions to "Completed"
- Business rules validation happens before order creation, preventing invalid orders
- The system gracefully handles missing business rules engine (backward compatible)
