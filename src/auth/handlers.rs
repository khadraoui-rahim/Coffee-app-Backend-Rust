// HTTP handlers for authentication endpoints

use axum::{extract::State, http::StatusCode, Json};
use crate::auth::{
    error::AuthError,
    models::{AuthResponse, LoginRequest, RefreshRequest, RegisterRequest, UserResponse},
};
use validator::Validate;

/// Register a new user
/// POST /api/auth/register
#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = AuthResponse),
        (status = 400, description = "Invalid input data", body = String),
        (status = 409, description = "Email already exists", body = String)
    ),
    tag = "auth"
)]
pub async fn register_handler(
    State(state): State<crate::AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AuthError> {
    // Validate request
    request.validate()
        .map_err(|e| AuthError::ValidationError(e.to_string()))?;
    
    // Register user
    let response = state.auth_service.register(&request.email, &request.password).await?;
    
    Ok((StatusCode::CREATED, Json(response)))
}

/// Login a user
/// POST /api/auth/login
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 400, description = "Invalid input data", body = String),
        (status = 401, description = "Invalid credentials", body = String)
    ),
    tag = "auth"
)]
pub async fn login_handler(
    State(state): State<crate::AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AuthError> {
    // Validate request
    request.validate()
        .map_err(|e| AuthError::ValidationError(e.to_string()))?;
    
    // Login user
    let response = state.auth_service.login(&request.email, &request.password).await?;
    
    Ok(Json(response))
}

/// Refresh tokens
/// POST /api/auth/refresh
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Tokens refreshed successfully", body = AuthResponse),
        (status = 401, description = "Invalid or expired refresh token", body = String)
    ),
    tag = "auth"
)]
pub async fn refresh_handler(
    State(state): State<crate::AppState>,
    Json(request): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, AuthError> {
    // Refresh tokens
    let response = state.auth_service.refresh_tokens(&request.refresh_token).await?;
    
    Ok(Json(response))
}

/// Get current user information (protected endpoint)
/// GET /api/auth/me
#[utoipa::path(
    get,
    path = "/api/auth/me",
    responses(
        (status = 200, description = "Current user information", body = UserResponse),
        (status = 401, description = "Unauthorized - invalid or missing token", body = String)
    ),
    tag = "auth",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn me_handler(
    State(state): State<crate::AppState>,
    user: crate::auth::middleware::AuthenticatedUser,
) -> Result<Json<UserResponse>, AuthError> {
    // Get current user
    let user_response = state.auth_service.get_current_user(user.user_id).await?;
    
    Ok(Json(user_response))
}
