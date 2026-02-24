// Authentication service - business logic layer

use crate::auth::{
    error::AuthError,
    models::{AuthResponse, User, UserResponse},
    password::PasswordService,
    repository::{TokenRepository, UserRepository},
    token::TokenService,
};
use chrono::Utc;
use tracing::info;

/// Authentication service coordinating all auth operations
pub struct AuthService {
    user_repo: UserRepository,
    token_repo: TokenRepository,
    token_service: TokenService,
}

impl AuthService {
    /// Create a new AuthService
    pub fn new(
        user_repo: UserRepository,
        token_repo: TokenRepository,
        _password_service: PasswordService,
        token_service: TokenService,
    ) -> Self {
        Self {
            user_repo,
            token_repo,
            token_service,
        }
    }

    /// Register a new user
    pub async fn register(&self, email: &str, password: &str) -> Result<AuthResponse, AuthError> {
        // Validate email format using regex
        let email_regex = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .map_err(|_| AuthError::ValidationError("Invalid email regex".to_string()))?;
        
        if !email_regex.is_match(email) {
            return Err(AuthError::ValidationError("Invalid email format".to_string()));
        }

        // Validate password strength
        PasswordService::validate_password_strength(password)?;

        // Check for duplicate email
        if self.user_repo.email_exists(email).await? {
            return Err(AuthError::EmailAlreadyExists);
        }

        // Hash password
        let password_hash = PasswordService::hash_password(password)?;

        // Create user
        let user = self.user_repo.create_user(email, &password_hash).await?;

        // Generate token pair
        let (access_token, refresh_token) = self.token_service.generate_token_pair(user.id, &user.email, user.role)?;

        // Calculate refresh token expiration (7 days from now)
        let refresh_expires_at = Utc::now() + chrono::Duration::days(7);

        // Store refresh token
        self.token_repo.store_refresh_token(user.id, &refresh_token, refresh_expires_at).await?;

        // Return response
        Ok(AuthResponse {
            access_token,
            refresh_token,
            user: user.into(),
        })
    }

    /// Login a user
    pub async fn login(&self, email: &str, password: &str) -> Result<AuthResponse, AuthError> {
        // Find user by email
        let user = self.user_repo.find_by_email(email).await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Verify password
        if !PasswordService::verify_password(password, &user.password_hash)? {
            return Err(AuthError::InvalidCredentials);
        }

        // Generate token pair
        let (access_token, refresh_token) = self.token_service.generate_token_pair(user.id, &user.email, user.role)?;

        // Calculate refresh token expiration (7 days from now)
        let refresh_expires_at = Utc::now() + chrono::Duration::days(7);

        // Store refresh token
        self.token_repo.store_refresh_token(user.id, &refresh_token, refresh_expires_at).await?;

        // Return response
        Ok(AuthResponse {
            access_token,
            refresh_token,
            user: user.into(),
        })
    }

    /// Refresh access and refresh tokens
    pub async fn refresh_tokens(&self, refresh_token: &str) -> Result<AuthResponse, AuthError> {
        // Validate refresh token
        let _claims = self.token_service.validate_refresh_token(refresh_token)?;

        // Verify refresh token exists in database
        let stored_token = self.token_repo.verify_refresh_token(refresh_token).await?
            .ok_or(AuthError::InvalidToken)?;

        // Get user information
        let user = self.user_repo.find_by_id(stored_token.user_id).await?
            .ok_or(AuthError::InvalidToken)?;

        // Invalidate old refresh token
        self.token_repo.invalidate_token(refresh_token).await?;

        // Generate new token pair
        let (new_access_token, new_refresh_token) = self.token_service.generate_token_pair(user.id, &user.email, user.role)?;

        // Calculate refresh token expiration (7 days from now)
        let refresh_expires_at = Utc::now() + chrono::Duration::days(7);

        // Store new refresh token
        self.token_repo.store_refresh_token(user.id, &new_refresh_token, refresh_expires_at).await?;

        // Return response
        Ok(AuthResponse {
            access_token: new_access_token,
            refresh_token: new_refresh_token,
            user: user.into(),
        })
    }

    /// Get current user information
    pub async fn get_current_user(&self, user_id: i32) -> Result<UserResponse, AuthError> {
        // Find user by ID
        let user = self.user_repo.find_by_id(user_id).await?
            .ok_or(AuthError::InvalidToken)?;

        // Convert to UserResponse (excluding password_hash)
        Ok(user.into())
    }

    /// Get user with role information
    pub async fn get_user_with_role(&self, user_id: i32) -> Result<(User, crate::auth::models::Role), AuthError> {
        // Find user by ID
        let user = self.user_repo.find_by_id(user_id).await?
            .ok_or_else(|| AuthError::DatabaseError("User not found".to_string()))?;

        // Return user and role tuple
        Ok((user.clone(), user.role))
    }

    /// Create a user with a specific role
    pub async fn create_user_with_role(
        &self,
        email: &str,
        password: &str,
        role: Option<crate::auth::models::Role>,
    ) -> Result<AuthResponse, AuthError> {
        // Validate email format using regex
        let email_regex = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .map_err(|_| AuthError::ValidationError("Invalid email regex".to_string()))?;
        
        if !email_regex.is_match(email) {
            return Err(AuthError::ValidationError("Invalid email format".to_string()));
        }

        // Validate password strength
        PasswordService::validate_password_strength(password)?;

        // Check for duplicate email
        if self.user_repo.email_exists(email).await? {
            return Err(AuthError::EmailAlreadyExists);
        }

        // Hash password
        let password_hash = PasswordService::hash_password(password)?;

        // Create user with specified role (or default User role)
        let user = self.user_repo.create_user_with_role(email, &password_hash, role).await?;

        // Log role assignment
        info!(
            "Role assigned: user_id={}, email={}, role={}",
            user.id, user.email, user.role
        );

        // Generate token pair
        let (access_token, refresh_token) = self.token_service.generate_token_pair(user.id, &user.email, user.role)?;

        // Calculate refresh token expiration (7 days from now)
        let refresh_expires_at = Utc::now() + chrono::Duration::days(7);

        // Store refresh token
        self.token_repo.store_refresh_token(user.id, &refresh_token, refresh_expires_at).await?;

        // Return response
        Ok(AuthResponse {
            access_token,
            refresh_token,
            user: user.into(),
        })
    }

    /// Update a user's role
    /// Requires admin permissions and prevents self-role-modification
    pub async fn update_user_role(
        &self,
        caller_id: i32,
        target_user_id: i32,
        new_role: crate::auth::models::Role,
    ) -> Result<UserResponse, AuthError> {
        use crate::auth::models::Role;
        use tracing::warn;

        // Get caller's information
        let caller = self.user_repo.find_by_id(caller_id).await?
            .ok_or_else(|| AuthError::DatabaseError("Caller not found".to_string()))?;

        // Verify caller is admin
        if caller.role != Role::Admin {
            warn!(
                "Failed role change attempt: caller_id={}, target_user_id={}, reason=insufficient_permissions",
                caller_id, target_user_id
            );
            return Err(AuthError::InsufficientPermissions {
                required: Role::Admin,
                actual: caller.role,
            });
        }

        // Prevent self-role-modification
        if caller_id == target_user_id {
            warn!(
                "Failed role change attempt: caller_id={}, target_user_id={}, reason=self_modification",
                caller_id, target_user_id
            );
            return Err(AuthError::ValidationError(
                "Cannot modify your own role".to_string()
            ));
        }

        // Validate new role is valid (already validated by type system, but check anyway)
        // This is redundant but satisfies requirement 7.4
        match new_role {
            Role::Admin | Role::User => {}, // Valid roles
        }

        // Get target user's current role for logging
        let target_user = self.user_repo.find_by_id(target_user_id).await?
            .ok_or_else(|| AuthError::DatabaseError("Target user not found".to_string()))?;
        let old_role = target_user.role;

        // Update role in database
        let updated_user = self.user_repo.update_user_role(target_user_id, new_role).await?;

        // Log successful role update
        info!(
            "Role updated: caller_id={}, target_user_id={}, old_role={}, new_role={}",
            caller_id, target_user_id, old_role, new_role
        );

        // Return updated user response
        Ok(updated_user.into())
    }
}
