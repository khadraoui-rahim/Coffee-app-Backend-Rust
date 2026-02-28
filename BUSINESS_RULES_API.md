# Business Rules System - API Documentation

This document provides comprehensive API documentation for the Business Rules System endpoints.

## Table of Contents

- [Authentication](#authentication)
- [Availability Management](#availability-management)
- [Pricing Rule Management](#pricing-rule-management)
- [Configuration Management](#configuration-management)
- [Performance Metrics](#performance-metrics)
- [Error Responses](#error-responses)

## Authentication

Most business rules management endpoints require authentication and admin privileges.

### Authentication Header

```
Authorization: Bearer <jwt_token>
```

### Required Roles

- **Admin**: Required for all management endpoints (POST, PUT, DELETE)
- **Public**: Read-only endpoints (GET) are publicly accessible

## Availability Management

### Update Coffee Availability

Updates the availability status of a coffee item.

**Endpoint:** `POST /api/business-rules/availability`

**Authentication:** Required (Admin only)

**Request Body:**

```json
{
  "coffee_id": 1,
  "status": "available",
  "available_from": "08:00",
  "available_until": "20:00"
}
```

**Request Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| coffee_id | integer | Yes | ID of the coffee item |
| status | string | Yes | One of: "available", "out_of_stock", "seasonal", "discontinued" |
| available_from | string | No | Start time in HH:MM format (24-hour) |
| available_until | string | No | End time in HH:MM format (24-hour) |

**Response:** `201 Created`

```json
{
  "coffee_id": 1,
  "status": "available",
  "available_from": "08:00",
  "available_until": "20:00",
  "updated_at": "2026-02-28T10:30:00Z"
}
```

**Example:**

```bash
curl -X POST http://localhost:8080/api/business-rules/availability \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "coffee_id": 1,
    "status": "available",
    "available_from": "08:00",
    "available_until": "20:00"
  }'
```

### Get Coffee Availability

Retrieves the availability status of a specific coffee item.

**Endpoint:** `GET /api/business-rules/availability/:coffee_id`

**Authentication:** Not required (Public)

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| coffee_id | integer | ID of the coffee item |

**Response:** `200 OK`

```json
{
  "coffee_id": 1,
  "status": "available",
  "available_from": "08:00",
  "available_until": "20:00",
  "updated_at": "2026-02-28T10:30:00Z"
}
```

**Example:**

```bash
curl http://localhost:8080/api/business-rules/availability/1
```

## Pricing Rule Management

### Create Pricing Rule

Creates a new pricing rule.

**Endpoint:** `POST /api/business-rules/pricing`

**Authentication:** Required (Admin only)

**Request Body:**

```json
{
  "rule_type": "time_based",
  "description": "Happy Hour - 20% off",
  "discount_type": "percentage",
  "discount_value": 20.0,
  "priority": 10,
  "valid_from": "2026-02-28T00:00:00Z",
  "valid_until": "2026-12-31T23:59:59Z",
  "coffee_ids": [1, 2, 3],
  "time_ranges": [
    {
      "start": "15:00",
      "end": "17:00"
    }
  ]
}
```

**Request Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| rule_type | string | Yes | One of: "time_based", "quantity_based", "promotional" |
| description | string | Yes | Human-readable description of the rule |
| discount_type | string | Yes | One of: "percentage", "fixed_amount" |
| discount_value | decimal | Yes | Discount value (e.g., 20 for 20% or $20) |
| priority | integer | Yes | Rule priority (higher = applied first) |
| valid_from | datetime | No | Rule start date/time (ISO 8601) |
| valid_until | datetime | No | Rule end date/time (ISO 8601) |
| coffee_ids | array | No | Specific coffee IDs (null = all coffees) |
| time_ranges | array | No | Time ranges for time_based rules |
| min_quantity | integer | No | Minimum quantity for quantity_based rules |

**Response:** `201 Created`

```json
{
  "id": 1,
  "rule_type": "time_based",
  "description": "Happy Hour - 20% off",
  "discount_type": "percentage",
  "discount_value": 20.0,
  "priority": 10,
  "is_active": true,
  "valid_from": "2026-02-28T00:00:00Z",
  "valid_until": "2026-12-31T23:59:59Z",
  "created_at": "2026-02-28T10:30:00Z"
}
```

**Example - Time-Based Rule:**

```bash
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
```

**Example - Quantity-Based Rule:**

```bash
curl -X POST http://localhost:8080/api/business-rules/pricing \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "rule_type": "quantity_based",
    "description": "Buy 3, get 10% off",
    "discount_type": "percentage",
    "discount_value": 10.0,
    "priority": 5,
    "min_quantity": 3
  }'
```

### Update Pricing Rule

Updates an existing pricing rule.

**Endpoint:** `PUT /api/business-rules/pricing/:rule_id`

**Authentication:** Required (Admin only)

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| rule_id | integer | ID of the pricing rule |

**Request Body:** Same as Create Pricing Rule

**Response:** `200 OK` (Same structure as Create response)

### Delete Pricing Rule

Deactivates a pricing rule (soft delete).

**Endpoint:** `DELETE /api/business-rules/pricing/:rule_id`

**Authentication:** Required (Admin only)

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| rule_id | integer | ID of the pricing rule |

**Response:** `204 No Content`

**Example:**

```bash
curl -X DELETE http://localhost:8080/api/business-rules/pricing/1 \
  -H "Authorization: Bearer <token>"
```

### List Pricing Rules

Retrieves all active pricing rules.

**Endpoint:** `GET /api/business-rules/pricing`

**Authentication:** Not required (Public)

**Response:** `200 OK`

```json
[
  {
    "id": 1,
    "rule_type": "time_based",
    "description": "Happy Hour - 20% off",
    "discount_type": "percentage",
    "discount_value": 20.0,
    "priority": 10,
    "is_active": true,
    "valid_from": "2026-02-28T00:00:00Z",
    "valid_until": "2026-12-31T23:59:59Z",
    "created_at": "2026-02-28T10:30:00Z"
  }
]
```

**Example:**

```bash
curl http://localhost:8080/api/business-rules/pricing
```

## Configuration Management

### Update Loyalty Configuration

Updates the loyalty program configuration.

**Endpoint:** `PUT /api/business-rules/loyalty-config`

**Authentication:** Required (Admin only)

**Request Body:**

```json
{
  "points_per_dollar": 1.5,
  "bonus_multipliers": {
    "1": 2.0,
    "5": 1.5
  }
}
```

**Request Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| points_per_dollar | decimal | Yes | Base points earned per dollar spent (must be >= 0) |
| bonus_multipliers | object | No | Coffee ID to multiplier mapping for bonus points |

**Response:** `200 OK`

```json
{
  "config_id": 1,
  "points_per_dollar": 1.5,
  "bonus_multipliers": {
    "1": 2.0,
    "5": 1.5
  },
  "updated_at": "2026-02-28T10:30:00Z"
}
```

**Example:**

```bash
curl -X PUT http://localhost:8080/api/business-rules/loyalty-config \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "points_per_dollar": 1.5,
    "bonus_multipliers": {"1": 2.0, "5": 1.5}
  }'
```

### Get Loyalty Configuration

Retrieves the current loyalty program configuration.

**Endpoint:** `GET /api/business-rules/loyalty-config`

**Authentication:** Not required (Public)

**Response:** `200 OK`

```json
{
  "config_id": 1,
  "points_per_dollar": 1.5,
  "bonus_multipliers": {
    "1": 2.0,
    "5": 1.5
  },
  "updated_at": "2026-02-28T10:30:00Z"
}
```

**Example:**

```bash
curl http://localhost:8080/api/business-rules/loyalty-config
```

### Update Prep Time Configuration

Updates the preparation time configuration for a specific coffee.

**Endpoint:** `PUT /api/business-rules/prep-time/:coffee_id`

**Authentication:** Required (Admin only)

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| coffee_id | integer | ID of the coffee item |

**Request Body:**

```json
{
  "base_minutes": 5,
  "per_additional_item": 2
}
```

**Request Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| base_minutes | integer | Yes | Base preparation time in minutes (must be >= 1) |
| per_additional_item | integer | Yes | Additional time per extra item in minutes (must be >= 0) |

**Response:** `204 No Content`

**Example:**

```bash
curl -X PUT http://localhost:8080/api/business-rules/prep-time/1 \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "base_minutes": 5,
    "per_additional_item": 2
  }'
```

## Performance Metrics

### Get Performance Metrics

Retrieves performance metrics for the business rules system.

**Endpoint:** `GET /api/business-rules/metrics`

**Authentication:** Not required (Public)

**Response:** `200 OK`

```json
{
  "cache": {
    "hit_rate": "95.2%",
    "hits": 1234,
    "misses": 62
  },
  "availability": {
    "checks": 856,
    "avg_time_ms": "12.34",
    "slow_operations": 2
  },
  "pricing": {
    "calculations": 856,
    "avg_time_ms": "45.67",
    "slow_operations": 5
  },
  "prep_time": {
    "estimates": 856,
    "avg_time_ms": "8.90",
    "slow_operations": 0
  },
  "loyalty": {
    "calculations": 234,
    "avg_time_ms": "15.23",
    "slow_operations": 1
  }
}
```

**Metrics Description:**

- **cache.hit_rate**: Percentage of cache hits (higher is better, target > 90%)
- **cache.hits**: Total number of cache hits
- **cache.misses**: Total number of cache misses
- **avg_time_ms**: Average execution time in milliseconds
- **slow_operations**: Count of operations exceeding 100ms threshold

**Example:**

```bash
curl http://localhost:8080/api/business-rules/metrics
```

## Error Responses

All endpoints return consistent error responses.

### Error Response Format

```json
{
  "error": "Error category",
  "details": "Detailed error message"
}
```

### HTTP Status Codes

| Status Code | Description |
|-------------|-------------|
| 200 OK | Request succeeded |
| 201 Created | Resource created successfully |
| 204 No Content | Request succeeded with no response body |
| 400 Bad Request | Invalid request data or validation error |
| 401 Unauthorized | Missing or invalid authentication token |
| 403 Forbidden | Insufficient permissions (not admin) |
| 404 Not Found | Resource not found |
| 500 Internal Server Error | Server error (database, calculation, etc.) |

### Common Error Examples

**Validation Error (400):**

```json
{
  "error": "Validation error",
  "details": "points_per_dollar: must be greater than or equal to 0"
}
```

**Authentication Error (401):**

```json
{
  "error": "Unauthorized",
  "details": "Missing authentication token"
}
```

**Permission Error (403):**

```json
{
  "error": "Forbidden",
  "details": "Insufficient permissions: required Admin, but user has User"
}
```

**Not Found Error (404):**

```json
{
  "error": "Coffee not found",
  "details": "Coffee not found: 999"
}
```

**Server Error (500):**

```json
{
  "error": "Database error",
  "details": "Database error: connection timeout"
}
```

## Integration with Orders

The business rules system is automatically integrated with the order creation and completion flow:

### Order Creation

When creating an order via `POST /api/orders`, the system automatically:

1. **Validates Availability**: Checks if all items are available
2. **Calculates Pricing**: Applies active pricing rules
3. **Estimates Prep Time**: Calculates preparation time based on items and queue

If any items are unavailable, the order creation fails with a 400 error:

```json
{
  "error": "Validation error",
  "details": "Items unavailable: 1: Out of stock, 3: Seasonal (not in season)"
}
```

### Order Completion

When an order status is updated to "Completed" via `PATCH /api/orders/:id/status`, the system automatically:

1. **Awards Loyalty Points**: Calculates and awards points to the customer
2. **Updates Balance**: Updates the customer's loyalty balance
3. **Logs Audit Trail**: Records the loyalty award in the audit log

## Rate Limiting

Currently, no rate limiting is implemented. Consider implementing rate limiting for production deployments to prevent abuse of management endpoints.

## Versioning

The API is currently unversioned. Future versions may use URL versioning (e.g., `/api/v2/business-rules/...`).

## Support

For issues or questions:
- Check the logs for detailed error messages
- Review the performance metrics endpoint for system health
- Consult the audit log for rule application history
