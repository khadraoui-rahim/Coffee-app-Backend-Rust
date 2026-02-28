use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;

/// Service-level errors for the reviews system
#[derive(Debug)]
pub enum ServiceError {
    /// Review not found
    NotFound,
    
    /// User has already reviewed this coffee
    DuplicateReview,
    
    /// User does not own this review
    Unauthorized,
    
    /// Validation error with details
    ValidationError(String),
    
    /// Coffee not found
    CoffeeNotFound,
    
    /// Database error
    DatabaseError(sqlx::Error),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::NotFound => write!(f, "Review not found"),
            ServiceError::DuplicateReview => {
                write!(f, "Duplicate review: user has already reviewed this coffee")
            }
            ServiceError::Unauthorized => {
                write!(f, "Unauthorized: user does not own this review")
            }
            ServiceError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ServiceError::CoffeeNotFound => write!(f, "Coffee not found"),
            ServiceError::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl std::error::Error for ServiceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ServiceError::DatabaseError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for ServiceError {
    fn from(err: sqlx::Error) -> Self {
        ServiceError::DatabaseError(err)
    }
}

/// Error response structure for API responses
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    pub fn new(error: String, message: String) -> Self {
        Self {
            error,
            message,
            details: None,
        }
    }

    pub fn with_details(error: String, message: String, details: serde_json::Value) -> Self {
        Self {
            error,
            message,
            details: Some(details),
        }
    }
}

/// Convert ServiceError to ErrorResponse
impl From<ServiceError> for ErrorResponse {
    fn from(err: ServiceError) -> Self {
        let (error_type, message) = match &err {
            ServiceError::NotFound => (
                "NOT_FOUND",
                "Review not found".to_string(),
            ),
            ServiceError::DuplicateReview => (
                "DUPLICATE_REVIEW",
                "User has already reviewed this coffee".to_string(),
            ),
            ServiceError::Unauthorized => (
                "FORBIDDEN",
                "User does not own this review".to_string(),
            ),
            ServiceError::ValidationError(msg) => (
                "VALIDATION_ERROR",
                msg.clone(),
            ),
            ServiceError::CoffeeNotFound => (
                "COFFEE_NOT_FOUND",
                "Coffee not found".to_string(),
            ),
            ServiceError::DatabaseError(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    "DATABASE_ERROR",
                    "An internal error occurred".to_string(),
                )
            }
        };

        ErrorResponse::new(error_type.to_string(), message)
    }
}

/// Convert ErrorResponse to HTTP response
impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let status = match self.error.as_str() {
            "NOT_FOUND" | "COFFEE_NOT_FOUND" => StatusCode::NOT_FOUND,
            "DUPLICATE_REVIEW" => StatusCode::CONFLICT,
            "FORBIDDEN" => StatusCode::FORBIDDEN,
            "VALIDATION_ERROR" => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(self)).into_response()
    }
}

/// Convert ServiceError to HTTP response
impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match self {
            ServiceError::NotFound => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                "Review not found".to_string(),
            ),
            ServiceError::DuplicateReview => (
                StatusCode::CONFLICT,
                "DUPLICATE_REVIEW",
                "User has already reviewed this coffee".to_string(),
            ),
            ServiceError::Unauthorized => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "User does not own this review".to_string(),
            ),
            ServiceError::ValidationError(msg) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                msg,
            ),
            ServiceError::CoffeeNotFound => (
                StatusCode::NOT_FOUND,
                "COFFEE_NOT_FOUND",
                "Coffee not found".to_string(),
            ),
            ServiceError::DatabaseError(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "An internal error occurred".to_string(),
                )
            }
        };

        let error_response = ErrorResponse::new(error_type.to_string(), message);
        (status, Json(error_response)).into_response()
    }
}
