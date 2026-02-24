// Password hashing and validation service

use crate::auth::error::AuthError;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Params,
};

/// Password service for hashing and verification
pub struct PasswordService;

impl PasswordService {
    /// Hash a password using Argon2id
    /// 
    /// Uses Argon2id algorithm with the following parameters:
    /// - Memory cost: 19456 KiB (19 MiB)
    /// - Time cost: 2 iterations
    /// - Parallelism: 1 thread
    /// - Salt: 16 bytes, randomly generated
    /// 
    /// Returns PHC string format hash
    pub fn hash_password(password: &str) -> Result<String, AuthError> {
        // Configure Argon2 parameters
        let params = Params::new(19456, 2, 1, None)
            .map_err(|_| AuthError::PasswordHashError)?;
        
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            params,
        );

        // Generate random salt
        let salt = SaltString::generate(&mut OsRng);

        // Hash password
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| AuthError::PasswordHashError)?
            .to_string();

        Ok(password_hash)
    }

    /// Verify a password against a hash
    /// 
    /// Uses constant-time comparison to prevent timing attacks
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|_| AuthError::InvalidToken)?;

        let argon2 = Argon2::default();

        // Verify returns Ok(()) on success, Err on failure
        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Validate password strength requirements
    /// 
    /// Requirements:
    /// - Minimum length: 8 characters
    /// - At least one uppercase letter (A-Z)
    /// - At least one lowercase letter (a-z)
    /// - At least one digit (0-9)
    pub fn validate_password_strength(password: &str) -> Result<(), AuthError> {
        let mut errors = Vec::new();

        // Check minimum length
        if password.len() < 8 {
            errors.push("Password must be at least 8 characters long");
        }

        // Check for uppercase letter
        if !password.chars().any(|c| c.is_uppercase()) {
            errors.push("Password must contain at least one uppercase letter");
        }

        // Check for lowercase letter
        if !password.chars().any(|c| c.is_lowercase()) {
            errors.push("Password must contain at least one lowercase letter");
        }

        // Check for digit
        if !password.chars().any(|c| c.is_numeric()) {
            errors.push("Password must contain at least one digit");
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AuthError::InvalidPasswordFormat(errors.join(", ")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_creates_valid_hash() {
        let password = "TestPassword123";
        let hash = PasswordService::hash_password(password).unwrap();
        
        // Verify hash starts with $argon2id$ prefix
        assert!(hash.starts_with("$argon2id$"));
    }

    #[test]
    fn test_verify_password_with_correct_password() {
        let password = "TestPassword123";
        let hash = PasswordService::hash_password(password).unwrap();
        
        let result = PasswordService::verify_password(password, &hash).unwrap();
        assert!(result);
    }

    #[test]
    fn test_verify_password_with_incorrect_password() {
        let password = "TestPassword123";
        let hash = PasswordService::hash_password(password).unwrap();
        
        let result = PasswordService::verify_password("WrongPassword123", &hash).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_unique_salt_per_password() {
        let password = "TestPassword123";
        let hash1 = PasswordService::hash_password(password).unwrap();
        let hash2 = PasswordService::hash_password(password).unwrap();
        
        // Same password should produce different hashes due to unique salts
        assert_ne!(hash1, hash2);
        
        // Both should verify correctly
        assert!(PasswordService::verify_password(password, &hash1).unwrap());
        assert!(PasswordService::verify_password(password, &hash2).unwrap());
    }

    #[test]
    fn test_validate_password_too_short() {
        let result = PasswordService::validate_password_strength("Test1");
        assert!(result.is_err());
        if let Err(AuthError::InvalidPasswordFormat(msg)) = result {
            assert!(msg.contains("at least 8 characters"));
        }
    }

    #[test]
    fn test_validate_password_no_uppercase() {
        let result = PasswordService::validate_password_strength("testpassword123");
        assert!(result.is_err());
        if let Err(AuthError::InvalidPasswordFormat(msg)) = result {
            assert!(msg.contains("uppercase letter"));
        }
    }

    #[test]
    fn test_validate_password_no_lowercase() {
        let result = PasswordService::validate_password_strength("TESTPASSWORD123");
        assert!(result.is_err());
        if let Err(AuthError::InvalidPasswordFormat(msg)) = result {
            assert!(msg.contains("lowercase letter"));
        }
    }

    #[test]
    fn test_validate_password_no_digit() {
        let result = PasswordService::validate_password_strength("TestPassword");
        assert!(result.is_err());
        if let Err(AuthError::InvalidPasswordFormat(msg)) = result {
            assert!(msg.contains("digit"));
        }
    }

    #[test]
    fn test_validate_password_valid() {
        let result = PasswordService::validate_password_strength("TestPassword123");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_password_multiple_errors() {
        let result = PasswordService::validate_password_strength("test");
        assert!(result.is_err());
        if let Err(AuthError::InvalidPasswordFormat(msg)) = result {
            // Should contain multiple error messages
            assert!(msg.contains("8 characters"));
            assert!(msg.contains("uppercase"));
            assert!(msg.contains("digit"));
        }
    }
}
