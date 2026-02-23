// Authentication service - business logic layer

use crate::auth::{
    error::AuthError,
    models::{AuthResponse, UserResponse},
    password::PasswordService,
    repository::{TokenRepository, UserRepository},
    token::TokenService,
};

/// Authentication service coordinating all auth operations
pub struct AuthService {
    user_repo: UserRepository,
    token_repo: TokenRepository,
    password_service: PasswordService,
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
            password_service: _password_service,
            token_service,
        }
    }

    /// Register a new user
    pub async fn register(&self, _email: &str, _password: &str) -> Result<AuthResponse, AuthError> {
        // TODO: Implement in Task 9.2
        todo!("Implement user registration")
    }

    /// Login a user
    pub async fn login(&self, _email: &str, _password: &str) -> Result<AuthResponse, AuthError> {
        // TODO: Implement in Task 9.3
        todo!("Implement user login")
    }

    /// Refresh access and refresh tokens
    pub async fn refresh_tokens(&self, _refresh_token: &str) -> Result<AuthResponse, AuthError> {
        // TODO: Implement in Task 9.4
        todo!("Implement token refresh")
    }

    /// Get current user information
    pub async fn get_current_user(&self, _user_id: i32) -> Result<UserResponse, AuthError> {
        // TODO: Implement in Task 9.5
        todo!("Implement get current user")
    }
}
