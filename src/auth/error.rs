// Authentication error types

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;

/// Authentication error types
#[derive(Debug)]
pub enum AuthError {
    ValidationError(String),
    InvalidCredentials,
    InvalidToken,
    ExpiredToken,
    MissingToken,
    EmailAlreadyExists,
    DatabaseError(String),
    PasswordHashError(String),
    TokenGenerationError(String),
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
            AuthError::PasswordHashError(msg) => write!(f, "Password hashing error: {}", msg),
            AuthError::TokenGenerationError(msg) => write!(f, "Token generation error: {}", msg),
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AuthError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid email or password".to_string())
            }
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token".to_string()),
            AuthError::ExpiredToken => (StatusCode::UNAUTHORIZED, "Token has expired".to_string()),
            AuthError::MissingToken => {
                (StatusCode::UNAUTHORIZED, "Missing authentication token".to_string())
            }
            AuthError::EmailAlreadyExists => {
                (StatusCode::CONFLICT, "Email already exists".to_string())
            }
            AuthError::DatabaseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AuthError::PasswordHashError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AuthError::TokenGenerationError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}
