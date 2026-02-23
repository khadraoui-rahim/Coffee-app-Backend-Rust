// Error handling module for the Coffee API
// Provides centralized error types and HTTP response conversion

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response, Json},
};
use serde::Serialize;
use chrono::Utc;
use tracing::{error, warn, debug};

/// Main error type for the API
/// All handlers should return Result<T, ApiError>
/// 
/// This enum represents all possible error types that can occur in the API.
/// Each variant maps to a specific HTTP status code and error response format.
#[derive(Debug)]
pub enum ApiError {
    /// Validation errors from request validation
    /// Maps to HTTP 400 Bad Request
    ValidationError(validator::ValidationErrors),
    
    /// Resource not found by ID
    /// Maps to HTTP 404 Not Found
    NotFound { 
        resource: String, 
        id: String 
    },
    
    /// Duplicate resource conflict
    /// Maps to HTTP 409 Conflict
    Conflict { 
        message: String 
    },
    
    /// Database operation errors
    /// Maps to HTTP 500 Internal Server Error
    /// Sensitive details are filtered from client responses
    DatabaseError(sqlx::Error),
    
    /// Internal server errors
    /// Maps to HTTP 500 Internal Server Error
    /// Sensitive details are filtered from client responses
    InternalError(String),
    
    /// Authentication failures
    /// Maps to HTTP 401 Unauthorized
    Unauthorized(String),
    
    /// Authorization failures
    /// Maps to HTTP 403 Forbidden
    Forbidden(String),
}

/// Consistent error response structure
/// 
/// This struct defines the JSON format for all error responses.
/// It ensures consistency across all error types and provides both
/// machine-readable (error_code) and human-readable (message) information.
/// 
/// Fields follow snake_case naming convention for consistency.
#[derive(Serialize)]
pub struct ErrorResponse {
    /// Machine-readable error code (e.g., "VALIDATION_ERROR", "NOT_FOUND")
    pub error_code: String,
    
    /// Human-readable error message
    pub message: String,
    
    /// Optional additional details (e.g., field-level validation errors)
    /// Omitted from JSON when None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    
    /// ISO 8601 timestamp of when the error occurred
    pub timestamp: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_response) = self.to_error_response();
        (status, Json(error_response)).into_response()
    }
}

impl ApiError {
    /// Convert ApiError to HTTP status code and ErrorResponse
    /// 
    /// This method handles the conversion of internal errors to client-facing responses.
    /// It includes appropriate logging at different levels based on error severity:
    /// - error!: For internal errors and database errors (500-level)
    /// - warn!: For client errors that might indicate issues (400-level)
    /// - debug!: For expected client errors (validation, not found)
    /// 
    /// Sensitive data is filtered from client responses to prevent information leakage.
    fn to_error_response(&self) -> (StatusCode, ErrorResponse) {
        match self {
            ApiError::ValidationError(errors) => {
                // Log validation errors at debug level (expected client errors)
                debug!("Validation error: {:?}", errors);
                
                (
                    StatusCode::BAD_REQUEST,
                    ErrorResponse {
                        error_code: "VALIDATION_ERROR".to_string(),
                        message: "Request validation failed".to_string(),
                        details: Some(serde_json::to_value(errors).unwrap_or(serde_json::json!({}))),
                        timestamp: Utc::now().to_rfc3339(),
                    }
                )
            }
            ApiError::NotFound { resource, id } => {
                // Log not found errors at debug level (expected client errors)
                debug!("Resource not found: {} with id {}", resource, id);
                
                (
                    StatusCode::NOT_FOUND,
                    ErrorResponse {
                        error_code: "NOT_FOUND".to_string(),
                        message: format!("{} with id {} not found", resource, id),
                        details: None,
                        timestamp: Utc::now().to_rfc3339(),
                    }
                )
            }
            ApiError::Conflict { message } => {
                // Log conflicts at warn level (might indicate data integrity issues)
                warn!("Conflict error: {}", message);
                
                (
                    StatusCode::CONFLICT,
                    ErrorResponse {
                        error_code: "CONFLICT".to_string(),
                        message: message.clone(),
                        details: None,
                        timestamp: Utc::now().to_rfc3339(),
                    }
                )
            }
            ApiError::DatabaseError(db_error) => {
                // Log the full database error internally at error level
                // This is critical for debugging but should not be exposed to clients
                error!("Database error: {:?}", db_error);
                
                // Return generic message to client (no sensitive data exposure)
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse {
                        error_code: "DATABASE_ERROR".to_string(),
                        message: "A database error occurred".to_string(),
                        details: None,
                        timestamp: Utc::now().to_rfc3339(),
                    }
                )
            }
            ApiError::InternalError(internal_msg) => {
                // Log the full internal error at error level
                // This is critical for debugging but should not be exposed to clients
                error!("Internal error: {}", internal_msg);
                
                // Return generic message to client (no sensitive data exposure)
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse {
                        error_code: "INTERNAL_ERROR".to_string(),
                        message: "An internal server error occurred".to_string(),
                        details: None,
                        timestamp: Utc::now().to_rfc3339(),
                    }
                )
            }
            ApiError::Unauthorized(message) => {
                // Log unauthorized attempts at warn level (security concern)
                warn!("Unauthorized access attempt: {}", message);
                
                (
                    StatusCode::UNAUTHORIZED,
                    ErrorResponse {
                        error_code: "UNAUTHORIZED".to_string(),
                        message: message.clone(),
                        details: None,
                        timestamp: Utc::now().to_rfc3339(),
                    }
                )
            }
            ApiError::Forbidden(message) => {
                // Log forbidden attempts at warn level (security concern)
                warn!("Forbidden access attempt: {}", message);
                
                (
                    StatusCode::FORBIDDEN,
                    ErrorResponse {
                        error_code: "FORBIDDEN".to_string(),
                        message: message.clone(),
                        details: None,
                        timestamp: Utc::now().to_rfc3339(),
                    }
                )
            }
        }
    }
    
    /// Get the HTTP status code for this error
    /// 
    /// This method provides a convenient way to get just the status code
    /// without building the full error response.
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
            ApiError::Conflict { .. } => StatusCode::CONFLICT,
            ApiError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
        }
    }
}

/// Convert sqlx errors to ApiError
impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        ApiError::DatabaseError(error)
    }
}

/// Convert validator errors to ApiError
impl From<validator::ValidationErrors> for ApiError {
    fn from(errors: validator::ValidationErrors) -> Self {
        ApiError::ValidationError(errors)
    }
}
