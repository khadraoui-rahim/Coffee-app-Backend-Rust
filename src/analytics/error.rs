// Analytics error handling
// Provides consistent error types and response formatting for analytics endpoints

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::analytics::types::{ApiResponse, ResponseMetadata};

/// Analytics-specific error types
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AnalyticsError {
    /// Validation error with field-level details
    ValidationError {
        field: String,
        value: String,
        expected: String,
        message: String,
    },
    /// Authentication error (missing or invalid token)
    AuthenticationError {
        message: String,
    },
    /// Authorization error (insufficient permissions)
    AuthorizationError {
        message: String,
        required_role: String,
    },
    /// Resource not found error
    NotFoundError {
        resource: String,
        identifier: String,
    },
    /// Timeout error (query took too long)
    TimeoutError {
        message: String,
        timeout_ms: u64,
    },
    /// Database error (sanitized for security)
    DatabaseError {
        message: String,
    },
    /// Generic internal error (sanitized for security)
    InternalError {
        message: String,
    },
}

impl AnalyticsError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            AnalyticsError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            AnalyticsError::AuthenticationError { .. } => StatusCode::UNAUTHORIZED,
            AnalyticsError::AuthorizationError { .. } => StatusCode::FORBIDDEN,
            AnalyticsError::NotFoundError { .. } => StatusCode::NOT_FOUND,
            AnalyticsError::TimeoutError { .. } => StatusCode::SERVICE_UNAVAILABLE,
            AnalyticsError::DatabaseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AnalyticsError::InternalError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get a user-friendly error message
    pub fn message(&self) -> String {
        match self {
            AnalyticsError::ValidationError { message, .. } => message.clone(),
            AnalyticsError::AuthenticationError { message } => message.clone(),
            AnalyticsError::AuthorizationError { message, .. } => message.clone(),
            AnalyticsError::NotFoundError { resource, identifier } => {
                format!("{} with identifier '{}' not found", resource, identifier)
            }
            AnalyticsError::TimeoutError { message, .. } => message.clone(),
            AnalyticsError::DatabaseError { message } => message.clone(),
            AnalyticsError::InternalError { message } => message.clone(),
        }
    }

    /// Convert to ApiResponse for consistent error responses
    pub fn to_response<T>(&self, metadata: ResponseMetadata) -> ApiResponse<T> {
        ApiResponse::error(self.message(), metadata)
    }

    /// Create a validation error
    pub fn validation(field: &str, value: &str, expected: &str) -> Self {
        AnalyticsError::ValidationError {
            field: field.to_string(),
            value: value.to_string(),
            expected: expected.to_string(),
            message: format!(
                "Invalid value '{}' for field '{}'. Expected: {}",
                value, field, expected
            ),
        }
    }

    /// Create an authentication error
    pub fn authentication(message: &str) -> Self {
        AnalyticsError::AuthenticationError {
            message: message.to_string(),
        }
    }

    /// Create an authorization error
    pub fn authorization(required_role: &str) -> Self {
        AnalyticsError::AuthorizationError {
            message: format!("Insufficient permissions. Required role: {}", required_role),
            required_role: required_role.to_string(),
        }
    }

    /// Create a not found error
    pub fn not_found(resource: &str, identifier: &str) -> Self {
        AnalyticsError::NotFoundError {
            resource: resource.to_string(),
            identifier: identifier.to_string(),
        }
    }

    /// Create a timeout error
    pub fn timeout(timeout_ms: u64) -> Self {
        AnalyticsError::TimeoutError {
            message: format!("Query exceeded timeout of {}ms", timeout_ms),
            timeout_ms,
        }
    }

    /// Create a database error (sanitized)
    pub fn database(message: &str) -> Self {
        AnalyticsError::DatabaseError {
            message: format!("Database error: {}", message),
        }
    }

    /// Create an internal error (sanitized)
    pub fn internal(message: &str) -> Self {
        AnalyticsError::InternalError {
            message: format!("Internal server error: {}", message),
        }
    }
}

/// Implement IntoResponse for AnalyticsError to use with Axum
impl IntoResponse for AnalyticsError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let metadata = ResponseMetadata::new(serde_json::json!({}));
        let response: ApiResponse<()> = self.to_response(metadata);
        
        (status, Json(response)).into_response()
    }
}

/// Error response with detailed validation information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ValidationErrorDetails {
    pub field: String,
    pub value: String,
    pub expected_format: String,
    pub message: String,
}

impl ValidationErrorDetails {
    pub fn new(field: &str, value: &str, expected_format: &str, message: &str) -> Self {
        Self {
            field: field.to_string(),
            value: value.to_string(),
            expected_format: expected_format.to_string(),
            message: message.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_creation() {
        let error = AnalyticsError::validation("startDate", "invalid", "ISO 8601 date format");
        
        match error {
            AnalyticsError::ValidationError { field, value, expected, message } => {
                assert_eq!(field, "startDate");
                assert_eq!(value, "invalid");
                assert_eq!(expected, "ISO 8601 date format");
                assert!(message.contains("startDate"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_validation_error_status_code() {
        let error = AnalyticsError::validation("limit", "-5", "positive integer");
        assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_authentication_error() {
        let error = AnalyticsError::authentication("Missing authorization token");
        assert_eq!(error.status_code(), StatusCode::UNAUTHORIZED);
        assert!(error.message().contains("authorization token"));
    }

    #[test]
    fn test_authorization_error() {
        let error = AnalyticsError::authorization("admin");
        assert_eq!(error.status_code(), StatusCode::FORBIDDEN);
        assert!(error.message().contains("admin"));
    }

    #[test]
    fn test_not_found_error() {
        let error = AnalyticsError::not_found("Coffee", "123");
        assert_eq!(error.status_code(), StatusCode::NOT_FOUND);
        assert!(error.message().contains("Coffee"));
        assert!(error.message().contains("123"));
    }

    #[test]
    fn test_timeout_error() {
        let error = AnalyticsError::timeout(5000);
        assert_eq!(error.status_code(), StatusCode::SERVICE_UNAVAILABLE);
        assert!(error.message().contains("5000"));
    }

    #[test]
    fn test_database_error() {
        let error = AnalyticsError::database("Connection failed");
        assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(error.message().contains("Database error"));
    }

    #[test]
    fn test_internal_error() {
        let error = AnalyticsError::internal("Unexpected condition");
        assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(error.message().contains("Internal server error"));
    }

    #[test]
    fn test_error_message_extraction() {
        let errors = vec![
            AnalyticsError::validation("field", "value", "format"),
            AnalyticsError::authentication("auth message"),
            AnalyticsError::not_found("Resource", "id"),
        ];

        for error in errors {
            let message = error.message();
            assert!(!message.is_empty());
        }
    }

    #[test]
    fn test_validation_error_details() {
        let details = ValidationErrorDetails::new(
            "limit",
            "1000",
            "integer between 1 and 100",
            "Limit exceeds maximum allowed value",
        );

        assert_eq!(details.field, "limit");
        assert_eq!(details.value, "1000");
        assert_eq!(details.expected_format, "integer between 1 and 100");
        assert!(details.message.contains("maximum"));
    }

    // Property 28: Validation error details - Include field, value, expected format
    #[test]
    fn test_validation_error_completeness() {
        let error = AnalyticsError::validation("period", "yearly", "daily, weekly, or monthly");
        
        match error {
            AnalyticsError::ValidationError { field, value, expected, message } => {
                // All required fields should be present
                assert!(!field.is_empty());
                assert!(!value.is_empty());
                assert!(!expected.is_empty());
                assert!(!message.is_empty());
                
                // Message should include field and value
                assert!(message.contains(&field));
                assert!(message.contains(&value));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_error_response_format() {
        let error = AnalyticsError::validation("test", "value", "format");
        let metadata = ResponseMetadata::new(serde_json::json!({}));
        let response: ApiResponse<()> = error.to_response(metadata);

        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
    }
}
