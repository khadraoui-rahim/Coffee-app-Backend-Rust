use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Error types for order operations
#[derive(Debug, thiserror::Error)]
pub enum OrderError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Order not found")]
    NotFound,

    #[error("Coffee item not found: {0}")]
    CoffeeNotFound(i32),

    #[error("Invalid quantity: {0}")]
    InvalidQuantity(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Invalid status transition: {0}")]
    InvalidTransition(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

impl From<sqlx::Error> for OrderError {
    fn from(err: sqlx::Error) -> Self {
        OrderError::DatabaseError(err.to_string())
    }
}

impl IntoResponse for OrderError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            OrderError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            OrderError::NotFound => (StatusCode::NOT_FOUND, "Order not found".to_string()),
            OrderError::CoffeeNotFound(id) => (
                StatusCode::BAD_REQUEST,
                format!("Coffee item with id {} not found", id),
            ),
            OrderError::InvalidQuantity(msg) => (StatusCode::BAD_REQUEST, msg),
            OrderError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            OrderError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            OrderError::InvalidTransition(msg) => (StatusCode::BAD_REQUEST, msg),
            OrderError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
