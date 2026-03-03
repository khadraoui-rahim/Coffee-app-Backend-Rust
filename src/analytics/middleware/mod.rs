// Analytics middleware
// Authentication and authorization middleware for admin-only access to analytics endpoints

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use crate::auth::{error::AuthError, middleware::RequireRole, models::Role};

/// Result type for authentication operations
#[derive(Debug)]
pub enum AuthResult {
    Allowed,
    Denied(AuthError),
}

impl AuthResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, AuthResult::Allowed)
    }
}

/// Analytics authentication middleware
/// Verifies that the user is authenticated and has admin role
pub struct AnalyticsAuthMiddleware;

impl AnalyticsAuthMiddleware {
    /// Verify admin access for analytics endpoints
    /// 
    /// This function validates:
    /// 1. User is authenticated (valid JWT token)
    /// 2. User has Admin role
    /// 
    /// Returns AuthResult::Allowed if both conditions are met,
    /// otherwise returns AuthResult::Denied with appropriate error
    pub async fn verify_admin_access(request: Request, next: Next) -> Response {
        // Use the existing RequireRole middleware to verify admin access
        let require_admin = RequireRole::admin();
        
        match require_admin.middleware(request, next).await {
            Ok(response) => response,
            Err(auth_error) => {
                // Convert AuthError to appropriate HTTP response
                auth_error.into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::token::TokenService;
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
        middleware,
        response::Response,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    // Helper to create a test token service
    fn test_token_service() -> TokenService {
        TokenService::new("test_secret_key_for_testing_purposes".to_string())
    }

    // Helper handler for testing
    async fn test_handler() -> &'static str {
        "success"
    }

    // Helper to create test app with analytics auth middleware
    fn create_test_app() -> Router {
        Router::new()
            .route("/analytics/test", get(test_handler))
            .layer(middleware::from_fn(AnalyticsAuthMiddleware::verify_admin_access))
    }

    // ============================================================================
    // Property 1: Non-admin rejection
    // ============================================================================

    #[tokio::test]
    async fn test_non_admin_user_rejected() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

        let service = test_token_service();
        let token = service.generate_access_token(1, "user@example.com", Role::User).unwrap();
        
        let app = create_test_app();
        
        let request = Request::builder()
            .uri("/analytics/test")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        
        // Should return 403 Forbidden for non-admin users
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_multiple_non_admin_users_rejected() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

        let service = test_token_service();
        let test_users = vec![
            (1, "user1@example.com"),
            (2, "user2@example.com"),
            (3, "user3@example.com"),
        ];

        for (user_id, email) in test_users {
            let token = service.generate_access_token(user_id, email, Role::User).unwrap();
            let app = create_test_app();
            
            let request = Request::builder()
                .uri("/analytics/test")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap();

            let response = app.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::FORBIDDEN);
        }
    }

    // ============================================================================
    // Property 2: Unauthenticated rejection
    // ============================================================================

    #[tokio::test]
    async fn test_unauthenticated_request_rejected() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

        let app = create_test_app();
        
        let request = Request::builder()
            .uri("/analytics/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        
        // Should return 401 Unauthorized for missing token
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_invalid_token_rejected() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

        let app = create_test_app();
        
        let request = Request::builder()
            .uri("/analytics/test")
            .header(header::AUTHORIZATION, "Bearer invalid_token_here")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        
        // Should return 401 Unauthorized for invalid token
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_malformed_authorization_header() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

        let malformed_headers = vec![
            "InvalidFormat token",
            "token_without_bearer",
            "Basic dXNlcjpwYXNz",
        ];

        for auth_value in malformed_headers {
            let app = create_test_app();
            
            let request = Request::builder()
                .uri("/analytics/test")
                .header(header::AUTHORIZATION, auth_value)
                .body(Body::empty())
                .unwrap();

            let response = app.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }
    }

    // ============================================================================
    // Property 3: Admin access granted
    // ============================================================================

    #[tokio::test]
    async fn test_admin_user_allowed() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

        let service = test_token_service();
        let token = service.generate_access_token(1, "admin@example.com", Role::Admin).unwrap();
        
        let app = create_test_app();
        
        let request = Request::builder()
            .uri("/analytics/test")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        
        // Should return 200 OK for admin users
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_multiple_admin_users_allowed() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

        let service = test_token_service();
        let test_admins = vec![
            (1, "admin1@example.com"),
            (2, "admin2@example.com"),
            (3, "admin3@example.com"),
        ];

        for (user_id, email) in test_admins {
            let token = service.generate_access_token(user_id, email, Role::Admin).unwrap();
            let app = create_test_app();
            
            let request = Request::builder()
                .uri("/analytics/test")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap();

            let response = app.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    #[tokio::test]
    async fn test_expired_token_rejected() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

        use jsonwebtoken::{encode, EncodingKey, Header};
        use crate::auth::token::Claims;
        use chrono::Utc;

        let claims = Claims {
            sub: 1,
            email: "admin@example.com".to_string(),
            role: Role::Admin,
            iat: Utc::now().timestamp() - 1000,
            exp: Utc::now().timestamp() - 500, // Expired 500 seconds ago
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("test_secret_key_for_testing_purposes".as_bytes()),
        ).unwrap();

        let app = create_test_app();
        
        let request = Request::builder()
            .uri("/analytics/test")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        
        // Should return 401 Unauthorized for expired token
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_missing_authorization_header() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

        let app = create_test_app();
        
        let request = Request::builder()
            .uri("/analytics/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        
        // Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // ============================================================================
    // Property-based tests using proptest
    // ============================================================================

    use proptest::prelude::*;

    proptest! {
        // Property: Any non-admin user should be rejected
        #[test]
        fn prop_non_admin_always_rejected(
            user_id in 1i32..1000000,
            email in "[a-z]{3,10}@[a-z]{3,10}\\.(com|org|net)"
        ) {
            std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

            let service = test_token_service();
            let token = service.generate_access_token(user_id, &email, Role::User)?;
            
            let rt = tokio::runtime::Runtime::new().unwrap();
            let app = create_test_app();
            
            let request = Request::builder()
                .uri("/analytics/test")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap();

            let response = rt.block_on(app.oneshot(request)).unwrap();
            
            prop_assert_eq!(response.status(), StatusCode::FORBIDDEN);
        }

        // Property: Any admin user should be allowed
        #[test]
        fn prop_admin_always_allowed(
            user_id in 1i32..1000000,
            email in "[a-z]{3,10}@[a-z]{3,10}\\.(com|org|net)"
        ) {
            std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

            let service = test_token_service();
            let token = service.generate_access_token(user_id, &email, Role::Admin)?;
            
            let rt = tokio::runtime::Runtime::new().unwrap();
            let app = create_test_app();
            
            let request = Request::builder()
                .uri("/analytics/test")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap();

            let response = rt.block_on(app.oneshot(request)).unwrap();
            
            prop_assert_eq!(response.status(), StatusCode::OK);
        }

        // Property: Any malformed token should be rejected
        #[test]
        fn prop_malformed_token_rejected(
            malformed in "[a-zA-Z0-9]{10,50}"
        ) {
            std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

            let rt = tokio::runtime::Runtime::new().unwrap();
            let app = create_test_app();
            
            let request = Request::builder()
                .uri("/analytics/test")
                .header(header::AUTHORIZATION, format!("Bearer {}", malformed))
                .body(Body::empty())
                .unwrap();

            let response = rt.block_on(app.oneshot(request)).unwrap();
            
            prop_assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }
    }
}
