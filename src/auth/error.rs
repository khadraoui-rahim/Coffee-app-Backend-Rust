// Authentication and authorization error types

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;
use tracing::{error, warn};
use crate::auth::models::Role;

/// Authentication and authorization error types
#[derive(Debug)]
pub enum AuthError {
    // Authentication errors
    ValidationError(String),
    InvalidCredentials,
    InvalidToken,
    ExpiredToken,
    MissingToken,
    EmailAlreadyExists,
    DatabaseError(String),
    PasswordHashError,
    InvalidPasswordFormat(String),
    TokenGenerationError(String),
    
    // Authorization errors
    /// User lacks required permissions for the operation
    /// Contains the required role and the user's actual role
    InsufficientPermissions {
        required: Role,
        actual: Role,
    },
    /// Invalid role value encountered
    InvalidRole(String),
    /// Configuration error in authorization system
    ConfigError(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AuthError::InvalidCredentials => write!(f, "Invalid email or password"),
            AuthError::InvalidToken => write!(f, "Invalid token"),
            AuthError::ExpiredToken => write!(f, "Token has expired"),
            AuthError::MissingToken => write!(f, "Missing authentication token"),
            AuthError::EmailAlreadyExists => write!(f, "Email already exists"),
            AuthError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AuthError::PasswordHashError => write!(f, "Password hashing error"),
            AuthError::InvalidPasswordFormat(msg) => write!(f, "Invalid password: {}", msg),
            AuthError::TokenGenerationError(msg) => write!(f, "Token generation error: {}", msg),
            AuthError::InsufficientPermissions { required, actual } => {
                write!(f, "Insufficient permissions: required role '{}', but user has role '{}'", required, actual)
            }
            AuthError::InvalidRole(msg) => write!(f, "Invalid role: {}", msg),
            AuthError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AuthError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AuthError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid email or password".to_string())
            }
            AuthError::InvalidToken => {
                warn!("Invalid token attempt");
                (StatusCode::UNAUTHORIZED, "Invalid token".to_string())
            }
            AuthError::ExpiredToken => {
                warn!("Expired token attempt");
                (StatusCode::UNAUTHORIZED, "Token has expired".to_string())
            }
            AuthError::MissingToken => {
                warn!("Missing token in request");
                (StatusCode::UNAUTHORIZED, "Missing authentication token".to_string())
            }
            AuthError::EmailAlreadyExists => {
                (StatusCode::CONFLICT, "Email already exists".to_string())
            }
            AuthError::DatabaseError(msg) => {
                error!("Database error in auth: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AuthError::PasswordHashError => {
                error!("Password hashing error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AuthError::InvalidPasswordFormat(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AuthError::TokenGenerationError(msg) => {
                error!("Token generation error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AuthError::InsufficientPermissions { required, actual } => {
                warn!("Authorization failed: required role '{}', user has role '{}'", required, actual);
                (
                    StatusCode::FORBIDDEN,
                    format!("Insufficient permissions: required role '{}'", required)
                )
            }
            AuthError::InvalidRole(msg) => {
                warn!("Invalid role encountered: {}", msg);
                (StatusCode::BAD_REQUEST, format!("Invalid role: {}", msg))
            }
            AuthError::ConfigError(msg) => {
                error!("Authorization configuration error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

impl AuthError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            AuthError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
            AuthError::ExpiredToken => StatusCode::UNAUTHORIZED,
            AuthError::MissingToken => StatusCode::UNAUTHORIZED,
            AuthError::EmailAlreadyExists => StatusCode::CONFLICT,
            AuthError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::PasswordHashError => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::InvalidPasswordFormat(_) => StatusCode::BAD_REQUEST,
            AuthError::TokenGenerationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::InsufficientPermissions { .. } => StatusCode::FORBIDDEN,
            AuthError::InvalidRole(_) => StatusCode::BAD_REQUEST,
            AuthError::ConfigError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    
    /// Get a descriptive error message for this error
    /// This message is safe to send to clients (no sensitive data)
    pub fn error_message(&self) -> String {
        match self {
            AuthError::ValidationError(msg) => msg.clone(),
            AuthError::InvalidCredentials => "Invalid email or password".to_string(),
            AuthError::InvalidToken => "Invalid token".to_string(),
            AuthError::ExpiredToken => "Token has expired".to_string(),
            AuthError::MissingToken => "Missing authentication token".to_string(),
            AuthError::EmailAlreadyExists => "Email already exists".to_string(),
            AuthError::DatabaseError(_) => "Internal server error".to_string(),
            AuthError::PasswordHashError => "Internal server error".to_string(),
            AuthError::InvalidPasswordFormat(msg) => msg.clone(),
            AuthError::TokenGenerationError(_) => "Internal server error".to_string(),
            AuthError::InsufficientPermissions { required, .. } => {
                format!("Insufficient permissions: required role '{}'", required)
            }
            AuthError::InvalidRole(msg) => format!("Invalid role: {}", msg),
            AuthError::ConfigError(_) => "Internal server error".to_string(),
        }
    }
}
