// JWT token generation and validation service

use crate::auth::error::AuthError;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,        // user_id
    pub email: String,
    pub exp: i64,        // expiration timestamp
    pub iat: i64,        // issued at timestamp
}

/// Token service for JWT operations
pub struct TokenService {
    secret: String,
    access_token_duration: i64,  // in seconds
    refresh_token_duration: i64, // in seconds
}

impl TokenService {
    /// Create a new TokenService with secret key
    /// Access tokens expire in 15 minutes (900 seconds)
    /// Refresh tokens expire in 7 days (604800 seconds)
    pub fn new(secret: String) -> Self {
        Self {
            secret,
            access_token_duration: 900,      // 15 minutes
            refresh_token_duration: 604800,  // 7 days
        }
    }

    /// Generate an access token (15 minutes)
    pub fn generate_access_token(&self, user_id: i32, email: &str) -> Result<String, AuthError> {
        let now = Utc::now().timestamp();
        let exp = now + self.access_token_duration;

        let claims = Claims {
            sub: user_id,
            email: email.to_string(),
            iat: now,
            exp,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AuthError::TokenGenerationError(e.to_string()))
    }

    /// Generate a refresh token (7 days)
    pub fn generate_refresh_token(&self, user_id: i32, email: &str) -> Result<String, AuthError> {
        let now = Utc::now().timestamp();
        let exp = now + self.refresh_token_duration;

        let claims = Claims {
            sub: user_id,
            email: email.to_string(),
            iat: now,
            exp,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AuthError::TokenGenerationError(e.to_string()))
    }

    /// Validate an access token
    pub fn validate_access_token(&self, token: &str) -> Result<Claims, AuthError> {
        self.validate_token(token)
    }

    /// Validate a refresh token
    pub fn validate_refresh_token(&self, token: &str) -> Result<Claims, AuthError> {
        self.validate_token(token)
    }

    /// Internal helper to validate any token
    fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        let validation = Validation::default();
        
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(|e| {
            // Check if the error is due to expiration
            if e.to_string().contains("ExpiredSignature") {
                AuthError::ExpiredToken
            } else {
                AuthError::InvalidToken
            }
        })
    }

    /// Generate both access and refresh tokens
    pub fn generate_token_pair(&self, user_id: i32, email: &str) -> Result<(String, String), AuthError> {
        let access_token = self.generate_access_token(user_id, email)?;
        let refresh_token = self.generate_refresh_token(user_id, email)?;
        Ok((access_token, refresh_token))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Helper to create a test token service
    fn test_token_service() -> TokenService {
        TokenService::new("test_secret_key_for_testing_purposes".to_string())
    }

    // Feature: authentication-system, Property 10: Access token expiration is 15 minutes
    #[test]
    fn test_access_token_expiration_is_15_minutes() {
        let service = test_token_service();
        let token = service.generate_access_token(1, "test@example.com").unwrap();
        let claims = service.validate_access_token(&token).unwrap();
        
        // Verify expiration is 15 minutes (900 seconds) from issued time
        let duration = claims.exp - claims.iat;
        assert_eq!(duration, 900, "Access token should expire in exactly 15 minutes (900 seconds)");
    }

    // Feature: authentication-system, Property 11: Refresh token expiration is 7 days
    #[test]
    fn test_refresh_token_expiration_is_7_days() {
        let service = test_token_service();
        let token = service.generate_refresh_token(1, "test@example.com").unwrap();
        let claims = service.validate_refresh_token(&token).unwrap();
        
        // Verify expiration is 7 days (604800 seconds) from issued time
        let duration = claims.exp - claims.iat;
        assert_eq!(duration, 604800, "Refresh token should expire in exactly 7 days (604800 seconds)");
    }

    // Feature: authentication-system, Property 12: Token claims contain user identity
    #[test]
    fn test_token_claims_contain_user_identity() {
        let service = test_token_service();
        let user_id = 42;
        let email = "user@example.com";
        
        let access_token = service.generate_access_token(user_id, email).unwrap();
        let access_claims = service.validate_access_token(&access_token).unwrap();
        assert_eq!(access_claims.sub, user_id);
        assert_eq!(access_claims.email, email);
        
        let refresh_token = service.generate_refresh_token(user_id, email).unwrap();
        let refresh_claims = service.validate_refresh_token(&refresh_token).unwrap();
        assert_eq!(refresh_claims.sub, user_id);
        assert_eq!(refresh_claims.email, email);
    }

    // Feature: authentication-system, Property 5: Successful registration returns token pair
    #[test]
    fn test_generate_token_pair() {
        let service = test_token_service();
        let (access_token, refresh_token) = service.generate_token_pair(1, "test@example.com").unwrap();
        
        // Both tokens should be valid
        assert!(service.validate_access_token(&access_token).is_ok());
        assert!(service.validate_refresh_token(&refresh_token).is_ok());
        
        // Tokens should be different
        assert_ne!(access_token, refresh_token);
    }

    // Feature: authentication-system, Property 15: Malformed tokens are rejected
    #[test]
    fn test_malformed_tokens_are_rejected() {
        let service = test_token_service();
        
        // Test various malformed tokens
        assert!(service.validate_access_token("").is_err());
        assert!(service.validate_access_token("not.a.token").is_err());
        assert!(service.validate_access_token("invalid_token_format").is_err());
        assert!(service.validate_access_token("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid.signature").is_err());
    }

    // Feature: authentication-system, Property 16: Token signature verification
    #[test]
    fn test_token_signature_verification() {
        let service1 = TokenService::new("secret1".to_string());
        let service2 = TokenService::new("secret2".to_string());
        
        // Generate token with service1
        let token = service1.generate_access_token(1, "test@example.com").unwrap();
        
        // service1 should validate it
        assert!(service1.validate_access_token(&token).is_ok());
        
        // service2 with different secret should reject it
        assert!(service2.validate_access_token(&token).is_err());
    }

    // Property-based tests using proptest

    proptest! {
        // Feature: authentication-system, Property 10: Access token expiration is 15 minutes
        #[test]
        fn prop_access_token_expiration(
            user_id in 1i32..1000000,
            email in "[a-z]{3,10}@[a-z]{3,10}\\.(com|org|net)"
        ) {
            let service = test_token_service();
            let token = service.generate_access_token(user_id, &email)?;
            let claims = service.validate_access_token(&token)?;
            
            let duration = claims.exp - claims.iat;
            prop_assert_eq!(duration, 900);
        }

        // Feature: authentication-system, Property 11: Refresh token expiration is 7 days
        #[test]
        fn prop_refresh_token_expiration(
            user_id in 1i32..1000000,
            email in "[a-z]{3,10}@[a-z]{3,10}\\.(com|org|net)"
        ) {
            let service = test_token_service();
            let token = service.generate_refresh_token(user_id, &email)?;
            let claims = service.validate_refresh_token(&token)?;
            
            let duration = claims.exp - claims.iat;
            prop_assert_eq!(duration, 604800);
        }

        // Feature: authentication-system, Property 12: Token claims contain user identity
        #[test]
        fn prop_token_claims_contain_identity(
            user_id in 1i32..1000000,
            email in "[a-z]{3,10}@[a-z]{3,10}\\.(com|org|net)"
        ) {
            let service = test_token_service();
            
            let access_token = service.generate_access_token(user_id, &email)?;
            let access_claims = service.validate_access_token(&access_token)?;
            prop_assert_eq!(access_claims.sub, user_id);
            prop_assert_eq!(access_claims.email, email.clone());
            
            let refresh_token = service.generate_refresh_token(user_id, &email)?;
            let refresh_claims = service.validate_refresh_token(&refresh_token)?;
            prop_assert_eq!(refresh_claims.sub, user_id);
            prop_assert_eq!(refresh_claims.email, email);
        }

        // Feature: authentication-system, Property 13: Valid access tokens are accepted
        #[test]
        fn prop_valid_tokens_are_accepted(
            user_id in 1i32..1000000,
            email in "[a-z]{3,10}@[a-z]{3,10}\\.(com|org|net)"
        ) {
            let service = test_token_service();
            
            let access_token = service.generate_access_token(user_id, &email)?;
            let result = service.validate_access_token(&access_token);
            prop_assert!(result.is_ok());
            
            let refresh_token = service.generate_refresh_token(user_id, &email)?;
            let result = service.validate_refresh_token(&refresh_token);
            prop_assert!(result.is_ok());
        }

        // Feature: authentication-system, Property 15: Malformed tokens are rejected
        #[test]
        fn prop_malformed_tokens_rejected(
            malformed in "[a-zA-Z0-9]{10,50}"
        ) {
            let service = test_token_service();
            
            // Random strings should be rejected as invalid tokens
            let result = service.validate_access_token(&malformed);
            prop_assert!(result.is_err());
        }
    }
}
