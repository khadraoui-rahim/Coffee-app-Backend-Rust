// Request logging middleware for analytics endpoints
// Logs all requests with query parameters and execution time
// Includes structured JSON logging and performance tracking

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::time::Instant;

/// Logging middleware for analytics requests
/// Logs request method, path, query parameters, and execution time
/// Includes structured JSON logging and performance tracking
pub async fn logging_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path();
    let query = uri.query().unwrap_or("");

    // Log the incoming request with structured JSON format
    tracing::info!(
        target: "analytics",
        method = %method,
        path = %path,
        query = %query,
        timestamp = %chrono::Utc::now().to_rfc3339(),
        event = "request_received",
        "Analytics request received"
    );

    // Process the request
    let response = next.run(request).await;

    // Calculate execution time
    let duration = start.elapsed();
    let duration_ms = duration.as_millis() as u64;
    let status = response.status();

    // Log the response with structured JSON format
    if status.is_success() {
        tracing::info!(
            target: "analytics",
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = duration_ms,
            timestamp = %chrono::Utc::now().to_rfc3339(),
            event = "request_completed",
            "Analytics request completed"
        );
    } else if status.is_client_error() {
        tracing::warn!(
            target: "analytics",
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = duration_ms,
            timestamp = %chrono::Utc::now().to_rfc3339(),
            event = "request_failed",
            error_type = "client_error",
            "Analytics request failed (client error)"
        );
    } else if status.is_server_error() {
        tracing::error!(
            target: "analytics",
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = duration_ms,
            timestamp = %chrono::Utc::now().to_rfc3339(),
            event = "request_failed",
            error_type = "server_error",
            "Analytics request failed (server error)"
        );
    }

    // Log slow queries (> 1 second)
    if duration.as_secs() >= 1 {
        tracing::warn!(
            target: "analytics",
            method = %method,
            path = %path,
            duration_ms = duration_ms,
            timestamp = %chrono::Utc::now().to_rfc3339(),
            event = "slow_query",
            "Slow analytics query detected"
        );
    }

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        response::IntoResponse,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn test_handler() -> impl IntoResponse {
        (StatusCode::OK, "test response")
    }

    #[tokio::test]
    async fn test_logging_middleware_success() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(logging_middleware));

        let request = Request::builder()
            .uri("/test?param=value")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_logging_middleware_with_query_params() {
        let app = Router::new()
            .route("/analytics/sales", get(test_handler))
            .layer(middleware::from_fn(logging_middleware));

        let request = Request::builder()
            .uri("/analytics/sales?startDate=2024-01-01&endDate=2024-01-31")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
