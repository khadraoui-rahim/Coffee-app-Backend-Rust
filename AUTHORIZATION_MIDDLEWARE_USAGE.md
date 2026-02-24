# Authorization Middleware Usage

## Overview

The `RequireRole` middleware provides role-based access control for protected routes. It validates JWT tokens and ensures users have the required role to access specific endpoints.

## Basic Usage

### Importing

```rust
use crate::auth::{RequireRole, Role};
use axum::{routing::post, Router};
```

### Protecting Routes

#### Require Admin Role

```rust
// Protect a route that requires admin access
let app = Router::new()
    .route("/admin/coffees", post(create_coffee_handler))
    .route_layer(axum::middleware::from_fn(
        RequireRole::admin().middleware
    ));
```

#### Require User Role

```rust
// Protect a route that requires user access
let app = Router::new()
    .route("/user/profile", post(update_profile_handler))
    .route_layer(axum::middleware::from_fn(
        RequireRole::user().middleware
    ));
```

#### Custom Role Requirement

```rust
// Protect a route with a specific role
let app = Router::new()
    .route("/protected", post(handler))
    .route_layer(axum::middleware::from_fn(
        RequireRole::new(Role::Admin).middleware
    ));
```

## How It Works

1. **Token Extraction**: The middleware extracts the JWT token from the `Authorization` header
2. **Token Validation**: Validates the token signature and expiration
3. **Role Extraction**: Extracts the user's role from the JWT claims
4. **Role Comparison**: Compares the user's role against the required role
5. **Access Decision**: 
   - If roles match → Request proceeds to the handler
   - If roles don't match → Returns 403 Forbidden error
   - If token is invalid/missing → Returns 401 Unauthorized error

## Error Responses

### Missing Token (401 Unauthorized)
```json
{
  "error": "Missing authentication token"
}
```

### Invalid Token (401 Unauthorized)
```json
{
  "error": "Invalid token"
}
```

### Expired Token (401 Unauthorized)
```json
{
  "error": "Token has expired"
}
```

### Insufficient Permissions (403 Forbidden)
```json
{
  "error": "Insufficient permissions: required role 'admin'"
}
```

## Example: Protecting Coffee Management Routes

```rust
use axum::{routing::{get, post, put, delete}, Router};
use crate::auth::RequireRole;

pub fn coffee_routes() -> Router {
    Router::new()
        // Public routes (no authentication required)
        .route("/coffees", get(list_coffees))
        .route("/coffees/:id", get(get_coffee))
        
        // Admin-only routes
        .route("/coffees", post(create_coffee))
        .route("/coffees/:id", put(update_coffee))
        .route("/coffees/:id", delete(delete_coffee))
        .route_layer(axum::middleware::from_fn(
            RequireRole::admin().middleware
        ))
}
```

## Testing

The middleware includes comprehensive unit tests covering:
- Valid tokens with matching roles
- Valid tokens with mismatched roles
- Missing Authorization headers
- Malformed Authorization headers
- Expired tokens
- Invalid token signatures

Run tests with:
```bash
docker compose exec backend cargo test --lib auth::middleware
```

## Logging

The middleware logs authorization events:
- **DEBUG**: Successful role validations
- **WARN**: Authorization failures (includes user_id, required_role, actual_role)

Example log output:
```
WARN Authorization failed: user_id=42, required_role=admin, actual_role=user
DEBUG Authorization successful: user_id=1, role=admin
```

## Security Considerations

1. **JWT Secret**: Ensure `JWT_SECRET` environment variable is set and kept secure
2. **Token Expiration**: Access tokens expire in 15 minutes
3. **Role Validation**: Roles are validated on every request
4. **Error Messages**: Error messages don't expose sensitive information
5. **Logging**: Authorization failures are logged for security monitoring
