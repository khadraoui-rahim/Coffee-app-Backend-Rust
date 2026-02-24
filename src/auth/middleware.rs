// Authentication middleware for protected routes

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use crate::auth::{error::AuthError, token::TokenService};

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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::token::TokenService;
    use axum::http::{Request, HeaderValue};
    use proptest::prelude::*;

    // Helper to create test parts with Authorization header
    fn create_parts_with_auth(auth_value: &str) -> Parts {
        let mut req = Request::builder()
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

    // Feature: authentication-system, Property 13: Valid access tokens are accepted
    #[tokio::test]
    async fn test_valid_token_is_accepted() {
        // Set JWT_SECRET for the test
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

        let service = test_token_service();
        let user_id = 42;
        let email = "test@example.com";
        
        let token = service.generate_access_token(user_id, email).unwrap();
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
            let token = service.generate_access_token(user_id, &email)?;
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
