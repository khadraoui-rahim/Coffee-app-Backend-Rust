// Database repositories for users and tokens

use crate::auth::{error::AuthError, models::{RefreshToken, User}};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

/// User repository for database operations
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    /// Create a new UserRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new user
    pub async fn create_user(&self, email: &str, password_hash: &str) -> Result<User, AuthError> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING id, email, password_hash, created_at"
        )
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            // Check for unique constraint violation
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return AuthError::EmailAlreadyExists;
                }
            }
            AuthError::DatabaseError(e.to_string())
        })?;

        Ok(user)
    }

    /// Find a user by email (case-insensitive)
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, AuthError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, created_at FROM users WHERE LOWER(email) = LOWER($1)"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(user)
    }

    /// Find a user by ID
    pub async fn find_by_id(&self, id: i32) -> Result<Option<User>, AuthError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, created_at FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(user)
    }

    /// Check if an email exists
    pub async fn email_exists(&self, email: &str) -> Result<bool, AuthError> {
        let exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM users WHERE LOWER(email) = LOWER($1))"
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(exists.0)
    }
}

/// Token repository for refresh token operations
pub struct TokenRepository {
    pool: PgPool,
}

impl TokenRepository {
    /// Create a new TokenRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Hash a token using SHA-256
    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Store a refresh token (hashed with SHA-256)
    pub async fn store_refresh_token(
        &self,
        user_id: i32,
        token: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), AuthError> {
        let token_hash = Self::hash_token(token);

        sqlx::query(
            "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)"
        )
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Verify a refresh token exists and is not expired
    pub async fn verify_refresh_token(&self, token: &str) -> Result<Option<RefreshToken>, AuthError> {
        let token_hash = Self::hash_token(token);

        let refresh_token = sqlx::query_as::<_, RefreshToken>(
            "SELECT id, user_id, token_hash, expires_at, created_at 
             FROM refresh_tokens 
             WHERE token_hash = $1 AND expires_at > NOW()"
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(refresh_token)
    }

    /// Invalidate a refresh token
    pub async fn invalidate_token(&self, token: &str) -> Result<(), AuthError> {
        let token_hash = Self::hash_token(token);

        sqlx::query("DELETE FROM refresh_tokens WHERE token_hash = $1")
            .bind(token_hash)
            .execute(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Delete expired tokens
    pub async fn delete_expired_tokens(&self) -> Result<u64, AuthError> {
        let result = sqlx::query("DELETE FROM refresh_tokens WHERE expires_at < NOW()")
            .execute(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected())
    }
}
