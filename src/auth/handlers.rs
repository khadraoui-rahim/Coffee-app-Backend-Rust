// HTTP handlers for authentication endpoints

use axum::{extract::State, http::StatusCode, Json};
use crate::auth::{
    error::AuthError,
    models::{AuthResponse, LoginRequest, RefreshRequest, RegisterRequest, UserResponse},
    service::AuthService,
};
use std::sync::Arc;

/// Register a new user
/// POST /api/auth/register
pub async fn register_handler(
    State(_service): State<Arc<AuthService>>,
    Json(_request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AuthError> {
    // TODO: Implement in Task 10.1
    todo!("Implement register handler")
}

/// Login a user
/// POST /api/auth/login
pub async fn login_handler(
    State(_service): State<Arc<AuthService>>,
    Json(_request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AuthError> {
    // TODO: Implement in Task 10.2
    todo!("Implement login handler")
}

/// Refresh tokens
/// POST /api/auth/refresh
pub async fn refresh_handler(
    State(_service): State<Arc<AuthService>>,
    Json(_request): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, AuthError> {
    // TODO: Implement in Task 10.3
    todo!("Implement refresh handler")
}

/// Get current user information (protected endpoint)
/// GET /api/auth/me
pub async fn me_handler(
    State(_service): State<Arc<AuthService>>,
    // _user: AuthenticatedUser, // TODO: Add in Task 10.4
) -> Result<Json<UserResponse>, AuthError> {
    // TODO: Implement in Task 10.4
    todo!("Implement me handler")
}
