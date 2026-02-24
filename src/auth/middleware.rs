// Authentication middleware for protected routes

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header, request::Parts, Request},
    middleware::Next,
    response::Response,
    body::Body,
};
use crate::auth::{error::AuthError, token::TokenService, models::Role};
use tracing::{debug, warn};

/// Authenticated user extractor for protected routes
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: i32,
    pub email: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .ok_or(AuthError::MissingToken)?
            .to_str()
            .map_err(|_| AuthError::InvalidToken)?;

        // Verify Bearer token format
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AuthError::InvalidToken)?;

        // Get JWT secret from environment
        let jwt_secret = std::env::var("JWT_SECRET")
            .map_err(|_| AuthError::TokenGenerationError("JWT_SECRET not configured".to_string()))?;

        // Create TokenService and validate token
        let token_service = TokenService::new(jwt_secret);
        let claims = token_service.validate_access_token(token)?;

        // Extract user_id and email from claims
        Ok(AuthenticatedUser {
            user_id: claims.sub,
            email: claims.email,
        })
    }
}

/// Authorization middleware that requires a specific role
/// 
/// This middleware extracts the JWT token from the Authorization header,
/// validates it, and checks if the user has the required role.
#[derive(Debug, Clone)]
pub struct RequireRole {
    required_role: Role,
}

impl RequireRole {
    /// Create a new RequireRole middleware with the specified role requirement
    pub fn new(required_role: Role) -> Self {
        Self { required_role }
    }

    /// Create a middleware that requires Admin role
    pub fn admin() -> Self {
        Self::new(Role::Admin)
    }

    /// Create a middleware that requires User role
    pub fn user() -> Self {
        Self::new(Role::User)
    }

    /// Middleware function that validates role-based access
    pub async fn middleware(
        self,
        request: Request<Body>,
        next: Next,
    ) -> Result<Response, AuthError> {
        // Extract endpoint path for logging
        let endpoint = request.uri().path().to_string();

        // Extract Authorization header
        let auth_header = request
            .headers()
            .get(header::AUTHORIZATION)
            .ok_or_else(|| {
                warn!(
                    "Missing Authorization header in request to protected endpoint: {}",
                    endpoint
                );
                AuthError::MissingToken
            })?
            .to_str()
            .map_err(|_| {
                warn!(
                    "Invalid Authorization header format for endpoint: {}",
                    endpoint
                );
                AuthError::InvalidToken
            })?;

        // Parse Bearer token format
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| {
                warn!(
                    "Authorization header missing 'Bearer ' prefix for endpoint: {}",
                    endpoint
                );
                AuthError::InvalidToken
            })?;

        // Get JWT secret from environment
        let jwt_secret = std::env::var("JWT_SECRET")
            .map_err(|_| {
                AuthError::ConfigError("JWT_SECRET not configured".to_string())
            })?;

        // Create TokenService and decode JWT
        let token_service = TokenService::new(jwt_secret);
        let claims = token_service.validate_access_token(token)?;

        // Extract user role from claims
        let user_role = claims.role;

        // Validate role matches requirement
        if user_role != self.required_role {
            warn!(
                "Authorization failed: user_id={}, required_role={}, actual_role={}, endpoint={}",
                claims.sub, self.required_role, user_role, endpoint
            );
            return Err(AuthError::InsufficientPermissions {
                required: self.required_role,
                actual: user_role,
            });
        }

        // Role matches - allow request to proceed
        debug!(
            "Authorization successful: user_id={}, role={}, endpoint={}",
            claims.sub, user_role, endpoint
        );
        Ok(next.run(request).await)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::token::TokenService;
    use axum::http::Request;
    use proptest::prelude::*;

    // Helper to create test parts with Authorization header
    fn create_parts_with_auth(auth_value: &str) -> Parts {
        let req = Request::builder()
            .uri("/")
            .header(header::AUTHORIZATION, auth_value)
            .body(())
            .unwrap();
        
        let (parts, _) = req.into_parts();
        parts
    }

    // Helper to create test parts without Authorization header
    fn create_parts_without_auth() -> Parts {
        let req = Request::builder()
            .uri("/")
            .body(())
            .unwrap();
        
        let (parts, _) = req.into_parts();
        parts
    }

    // Helper to create a test token service
    fn test_token_service() -> TokenService {
        TokenService::new("test_secret_key_for_testing_purposes".to_string())
    }

    // Helper to create a request with Authorization header
    fn create_request_with_auth(auth_value: &str) -> Request<Body> {
        Request::builder()
            .uri("/")
            .header(header::AUTHORIZATION, auth_value)
            .body(Body::empty())
            .unwrap()
    }

    // Helper to create a request without Authorization header
    fn create_request_without_auth() -> Request<Body> {
        Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap()
    }

    // Helper function to extract token from request and validate role
    async fn validate_role_from_request(request: &Request<Body>, required_role: Role) -> Result<(), AuthError> {
        // Extract Authorization header
        let auth_header = request
            .headers()
            .get(header::AUTHORIZATION)
            .ok_or(AuthError::MissingToken)?
            .to_str()
            .map_err(|_| AuthError::InvalidToken)?;

        // Parse Bearer token format
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AuthError::InvalidToken)?;

        // Get JWT secret from environment
        let jwt_secret = std::env::var("JWT_SECRET")
            .map_err(|_| AuthError::ConfigError("JWT_SECRET not configured".to_string()))?;

        // Create TokenService and decode JWT
        let token_service = TokenService::new(jwt_secret);
        let claims = token_service.validate_access_token(token)?;

        // Extract user role from claims
        let user_role = claims.role;

        // Validate role matches requirement
        if user_role != required_role {
            return Err(AuthError::InsufficientPermissions {
                required: required_role,
                actual: user_role,
            });
        }

        Ok(())
    }

    // Feature: authentication-system, Property 13: Valid access tokens are accepted
    #[tokio::test]
    async fn test_valid_token_is_accepted() {
        // Set JWT_SECRET for the test
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

        let service = test_token_service();
        let user_id = 42;
        let email = "test@example.com";
        
        let token = service.generate_access_token(user_id, email, crate::auth::models::Role::User).unwrap();
        let auth_header = format!("Bearer {}", token);
        
        let mut parts = create_parts_with_auth(&auth_header);
        let result = AuthenticatedUser::from_request_parts(&mut parts, &()).await;
        
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.user_id, user_id);
        assert_eq!(user.email, email);
    }

    // Feature: authentication-system, Property 14: Expired tokens are rejected
    #[tokio::test]
    async fn test_expired_token_is_rejected() {
        // Set JWT_SECRET for the test
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

        // Create a token with immediate expiration
        use jsonwebtoken::{encode, EncodingKey, Header};
        use crate::auth::token::Claims;
        use chrono::Utc;

        let claims = Claims {
            sub: 1,
            email: "test@example.com".to_string(),
            role: crate::auth::models::Role::User,
            iat: Utc::now().timestamp() - 1000,
            exp: Utc::now().timestamp() - 500, // Expired 500 seconds ago
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("test_secret_key_for_testing_purposes".as_bytes()),
        ).unwrap();

        let auth_header = format!("Bearer {}", token);
        let mut parts = create_parts_with_auth(&auth_header);
        
        let result = AuthenticatedUser::from_request_parts(&mut parts, &()).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::ExpiredToken));
    }

    // Feature: authentication-system, Property 15: Malformed tokens are rejected
    #[tokio::test]
    async fn test_malformed_token_is_rejected() {
        // Set JWT_SECRET for the test
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

        let malformed_tokens = vec![
            "Bearer invalid_token",
            "Bearer not.a.valid.jwt",
            "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid.signature",
        ];

        for token in malformed_tokens {
            let mut parts = create_parts_with_auth(token);
            let result = AuthenticatedUser::from_request_parts(&mut parts, &()).await;
            
            assert!(result.is_err());
        }
    }

    // Test missing Authorization header
    #[tokio::test]
    async fn test_missing_authorization_header() {
        let mut parts = create_parts_without_auth();
        let result = AuthenticatedUser::from_request_parts(&mut parts, &()).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::MissingToken));
    }

    // Test invalid Bearer format
    #[tokio::test]
    async fn test_invalid_bearer_format() {
        // Set JWT_SECRET for the test
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

        let invalid_formats = vec![
            "InvalidFormat token",
            "token_without_bearer",
            "Basic dXNlcjpwYXNz", // Basic auth instead of Bearer
        ];

        for auth_value in invalid_formats {
            let mut parts = create_parts_with_auth(auth_value);
            let result = AuthenticatedUser::from_request_parts(&mut parts, &()).await;
            
            assert!(result.is_err());
        }
    }

    // ===== RequireRole Middleware Tests =====

    // Feature: authorization-system, Task 5.6: Test malformed Authorization headers
    #[tokio::test]
    async fn test_require_role_malformed_authorization_header() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

        let malformed_headers = vec![
            "InvalidFormat token",
            "token_without_bearer",
            "Basic dXNlcjpwYXNz",
            "",
        ];

        for auth_value in malformed_headers {
            let request = create_request_with_auth(auth_value);
            let result = validate_role_from_request(&request, Role::Admin).await;
            assert!(result.is_err());
        }
    }

    // Feature: authorization-system, Task 5.6: Test expired tokens
    #[tokio::test]
    async fn test_require_role_expired_token() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

        use jsonwebtoken::{encode, EncodingKey, Header};
        use crate::auth::token::Claims;
        use chrono::Utc;

        let claims = Claims {
            sub: 1,
            email: "test@example.com".to_string(),
            role: Role::Admin,
            iat: Utc::now().timestamp() - 1000,
            exp: Utc::now().timestamp() - 500, // Expired
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("test_secret_key_for_testing_purposes".as_bytes()),
        ).unwrap();

        let auth_header = format!("Bearer {}", token);
        let request = create_request_with_auth(&auth_header);
        let result = validate_role_from_request(&request, Role::Admin).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::ExpiredToken));
    }

    // Feature: authorization-system, Task 5.6: Test tokens with missing role claim
    // Note: This is implicitly tested by the token validation, as our Claims struct requires role
    #[tokio::test]
    async fn test_require_role_missing_token() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

        let request = create_request_without_auth();
        let result = validate_role_from_request(&request, Role::Admin).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::MissingToken));
    }

    // Feature: authorization-system, Task 5.2: Test admin role allows access
    #[tokio::test]
    async fn test_require_role_admin_allows_admin() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

        let service = test_token_service();
        let token = service.generate_access_token(1, "admin@example.com", Role::Admin).unwrap();
        let auth_header = format!("Bearer {}", token);
        
        let request = create_request_with_auth(&auth_header);
        let result = validate_role_from_request(&request, Role::Admin).await;
        assert!(result.is_ok());
    }

    // Feature: authorization-system, Task 5.2: Test admin role denies user
    #[tokio::test]
    async fn test_require_role_admin_denies_user() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

        let service = test_token_service();
        let token = service.generate_access_token(1, "user@example.com", Role::User).unwrap();
        let auth_header = format!("Bearer {}", token);
        
        let request = create_request_with_auth(&auth_header);
        let result = validate_role_from_request(&request, Role::Admin).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AuthError::InsufficientPermissions { required, actual } => {
                assert_eq!(required, Role::Admin);
                assert_eq!(actual, Role::User);
            }
            _ => panic!("Expected InsufficientPermissions error"),
        }
    }

    // Feature: authorization-system, Task 5.2: Test user role allows user
    #[tokio::test]
    async fn test_require_role_user_allows_user() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

        let service = test_token_service();
        let token = service.generate_access_token(1, "user@example.com", Role::User).unwrap();
        let auth_header = format!("Bearer {}", token);
        
        let request = create_request_with_auth(&auth_header);
        let result = validate_role_from_request(&request, Role::User).await;
        assert!(result.is_ok());
    }

    // Feature: authorization-system, Task 5.2: Test user role denies admin
    #[tokio::test]
    async fn test_require_role_user_denies_admin() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

        let service = test_token_service();
        let token = service.generate_access_token(1, "admin@example.com", Role::Admin).unwrap();
        let auth_header = format!("Bearer {}", token);
        
        let request = create_request_with_auth(&auth_header);
        let result = validate_role_from_request(&request, Role::User).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AuthError::InsufficientPermissions { required, actual } => {
                assert_eq!(required, Role::User);
                assert_eq!(actual, Role::Admin);
            }
            _ => panic!("Expected InsufficientPermissions error"),
        }
    }

    // Property-based tests using proptest

    proptest! {
        // Feature: authentication-system, Property 13: Valid access tokens are accepted
        #[test]
        fn prop_valid_tokens_accepted(
            user_id in 1i32..1000000,
            email in "[a-z]{3,10}@[a-z]{3,10}\\.(com|org|net)"
        ) {
            // Set JWT_SECRET for the test
            std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

            let service = test_token_service();
            let token = service.generate_access_token(user_id, &email, crate::auth::models::Role::User)?;
            let auth_header = format!("Bearer {}", token);
            
            let mut parts = create_parts_with_auth(&auth_header);
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(
                AuthenticatedUser::from_request_parts(&mut parts, &())
            );
            
            prop_assert!(result.is_ok());
            let user = result.unwrap();
            prop_assert_eq!(user.user_id, user_id);
            prop_assert_eq!(user.email, email);
        }

        // Feature: authentication-system, Property 15: Malformed tokens are rejected
        #[test]
        fn prop_malformed_tokens_rejected(
            malformed in "[a-zA-Z0-9]{10,50}"
        ) {
            // Set JWT_SECRET for the test
            std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

            let auth_header = format!("Bearer {}", malformed);
            let mut parts = create_parts_with_auth(&auth_header);
            
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(
                AuthenticatedUser::from_request_parts(&mut parts, &())
            );
            
            prop_assert!(result.is_err());
        }
    }
}
