// Error types for the Business Rules System
// Provides comprehensive error handling for rule evaluation and configuration

use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Main error type for the Business Rules System
/// 
/// This enum represents all possible error types that can occur during
/// business rule evaluation, configuration loading, and rule application.
#[derive(Debug, Error)]
pub enum BusinessRulesError {
    /// Validation errors when checking order items against availability rules
    /// Contains a descriptive message about what validation failed
    #[error("Validation failed: {0}")]
    ValidationError(String),
    
    /// Specific error for unavailable coffee items
    /// Contains the coffee ID and reason for unavailability
    #[error("Coffee item {coffee_id} is unavailable: {reason}")]
    UnavailableItem {
        coffee_id: i32,
        reason: String,
    },
    
    /// Invalid pricing rule configuration
    /// Occurs when a pricing rule has invalid JSON structure or values
    #[error("Invalid pricing rule configuration: {0}")]
    InvalidPricingRule(String),
    
    /// Invalid configuration for any rule type
    /// Occurs when configuration fails validation checks
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    /// Database operation errors
    /// Automatically converted from sqlx::Error
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    /// Configuration not found in database
    /// Occurs when required configuration is missing
    #[error("Configuration not found: {0}")]
    ConfigurationNotFound(String),
    
    /// Calculation errors during rule evaluation
    /// Occurs when mathematical operations fail or produce invalid results
    #[error("Calculation error: {0}")]
    CalculationError(String),
    
    /// JSON serialization/deserialization errors
    /// Occurs when parsing rule configurations from JSONB
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    /// Coffee item not found in database
    /// Occurs when referencing a non-existent coffee
    #[error("Coffee not found: {0}")]
    CoffeeNotFound(i32),
    
    /// User not found in database
    /// Occurs when referencing a non-existent user for loyalty operations
    #[error("User not found: {0}")]
    UserNotFound(i32),
    
    /// Order not found in database
    /// Occurs when referencing a non-existent order
    #[error("Order not found: {0}")]
    OrderNotFound(String),
}

/// Result type alias for Business Rules operations
/// 
/// This type alias simplifies function signatures throughout the business rules system.
/// Instead of writing `Result<T, BusinessRulesError>`, you can write `BRResult<T>`.
pub type BRResult<T> = Result<T, BusinessRulesError>;

impl From<validator::ValidationErrors> for BusinessRulesError {
    fn from(err: validator::ValidationErrors) -> Self {
        BusinessRulesError::ValidationError(err.to_string())
    }
}

impl IntoResponse for BusinessRulesError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            BusinessRulesError::ValidationError(_) => {
                (StatusCode::BAD_REQUEST, "Validation error")
            }
            BusinessRulesError::UnavailableItem { .. } => {
                (StatusCode::BAD_REQUEST, "Item unavailable")
            }
            BusinessRulesError::InvalidPricingRule(_) => {
                (StatusCode::BAD_REQUEST, "Invalid pricing rule")
            }
            BusinessRulesError::InvalidConfiguration(_) => {
                (StatusCode::BAD_REQUEST, "Invalid configuration")
            }
            BusinessRulesError::DatabaseError(ref e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            }
            BusinessRulesError::ConfigurationNotFound(_) => {
                (StatusCode::NOT_FOUND, "Configuration not found")
            }
            BusinessRulesError::CalculationError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Calculation error")
            }
            BusinessRulesError::JsonError(_) => {
                (StatusCode::BAD_REQUEST, "JSON parsing error")
            }
            BusinessRulesError::CoffeeNotFound(_) => {
                (StatusCode::NOT_FOUND, "Coffee not found")
            }
            BusinessRulesError::UserNotFound(_) => {
                (StatusCode::NOT_FOUND, "User not found")
            }
            BusinessRulesError::OrderNotFound(_) => {
                (StatusCode::NOT_FOUND, "Order not found")
            }
        };

        let body = Json(json!({
            "error": error_message,
            "details": self.to_string(),
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = BusinessRulesError::ValidationError("test validation".to_string());
        assert_eq!(error.to_string(), "Validation failed: test validation");
        
        let error = BusinessRulesError::UnavailableItem {
            coffee_id: 1,
            reason: "out of stock".to_string(),
        };
        assert_eq!(error.to_string(), "Coffee item 1 is unavailable: out of stock");
        
        let error = BusinessRulesError::InvalidPricingRule("invalid discount".to_string());
        assert_eq!(error.to_string(), "Invalid pricing rule configuration: invalid discount");
    }
    
    #[test]
    fn test_error_from_sqlx() {
        // Test that sqlx::Error can be converted to BusinessRulesError
        let sqlx_error = sqlx::Error::RowNotFound;
        let br_error: BusinessRulesError = sqlx_error.into();
        assert!(matches!(br_error, BusinessRulesError::DatabaseError(_)));
    }
    
    #[test]
    fn test_error_from_json() {
        // Test that serde_json::Error can be converted to BusinessRulesError
        let json_str = "{invalid json}";
        let json_result: Result<serde_json::Value, _> = serde_json::from_str(json_str);
        
        if let Err(json_error) = json_result {
            let br_error: BusinessRulesError = json_error.into();
            assert!(matches!(br_error, BusinessRulesError::JsonError(_)));
        }
    }
}
