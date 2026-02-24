# Authorization System - Coffee Routes Protection

## Overview

Task 7.1 of the authorization system has been implemented. The coffee management routes are now protected with role-based access control using the `RequireRole` middleware.

## Protected Routes (Admin Only)

The following routes now require an Admin role:

1. **POST /api/coffees** - Create a new coffee
   - Requires: Bearer token with Admin role
   - Returns 401 if token is missing or invalid
   - Returns 403 if user has insufficient permissions (User role)

2. **PUT /api/coffees/:id** - Update an existing coffee
   - Requires: Bearer token with Admin role
   - Returns 401 if token is missing or invalid
   - Returns 403 if user has insufficient permissions (User role)

3. **DELETE /api/coffees/:id** - Delete a coffee
   - Requires: Bearer token with Admin role
   - Returns 401 if token is missing or invalid
   - Returns 403 if user has insufficient permissions (User role)

## Public Routes (No Authorization)

The following routes remain public and do not require authentication:

1. **GET /api/coffees** - List all coffees (with query parameters)
2. **GET /api/coffees/:id** - Get a specific coffee by ID

## Implementation Details

### Router Structure

The router is organized into two groups:

```rust
// Protected admin routes with RequireRole middleware
let admin_routes = Router::new()
    .route("/api/coffees", post(create_coffee))
    .route("/api/coffees/:id", put(update_coffee))
    .route("/api/coffees/:id", delete(delete_coffee))
    .route_layer(from_fn(move |req, next| {
        auth::middleware::RequireRole::admin().middleware(req, next)
    }));

// Public routes (no authorization required)
let public_routes = Router::new()
    .route("/api/coffees", get(get_coffees_with_query))
    .route("/api/coffees/:id", get(get_coffee_by_id))
    .route("/api/coffees/favorites/:id", get(get_favorite_coffee));
```

### Middleware Behavior

The `RequireRole::admin()` middleware:
1. Extracts the JWT token from the Authorization header
2. Validates the token signature and expiration
3. Extracts the user's role from the token claims
4. Compares the user's role against the required role (Admin)
5. Returns 403 Forbidden if roles don't match
6. Allows the request to proceed if the role matches

### OpenAPI Documentation

The protected endpoints have been updated with:
- `security(("bearer_auth" = []))` annotation
- Additional response codes:
  - 401: Unauthorized - Missing or invalid token
  - 403: Forbidden - Insufficient permissions

## Testing

To test the authorization:

1. **Create an admin user** (requires direct database access or a separate admin creation endpoint)
2. **Login as admin** to get a JWT token with Admin role
3. **Use the token** in the Authorization header: `Bearer <token>`
4. **Try to access protected routes** with and without the token

### Example cURL Commands

```bash
# Login as admin
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"AdminPass123"}'

# Create coffee (requires admin token)
curl -X POST http://localhost:8080/api/coffees \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <admin_token>" \
  -d '{"name":"Espresso","coffee_type":"Dark","price":3.50,"rating":4.5,"image_url":"https://example.com/image.jpg"}'

# List coffees (no token required)
curl http://localhost:8080/api/coffees
```

## Requirements Satisfied

This implementation satisfies the following requirements from the authorization system specification:

- **4.2**: Protected routes enforce authorization before handler execution
- **4.3**: Create coffee endpoint requires Admin role
- **4.4**: Update coffee endpoint requires Admin role
- **4.5**: Delete coffee endpoint requires Admin role

## Next Steps

The following tasks remain for complete authorization system implementation:

- Task 7.2: Property test for admin coffee management permissions
- Task 7.3: Property test for regular user coffee management denial
- Task 7.4: Property test for protected routes enforce authorization
- Task 8: Implement logging for authorization events
- Task 9: Integration testing and final validation
