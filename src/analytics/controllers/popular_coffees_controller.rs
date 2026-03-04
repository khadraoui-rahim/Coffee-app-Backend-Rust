// Popular coffees API controller
// Handles endpoints for most ordered, highest rated, and trending coffees

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;

use crate::analytics::{
    services::{PopularCoffeesService, TrendCalculationService},
    types::{ApiResponse, ResponseMetadata, PopularCoffee},
    utils::TimePeriodFilter,
};

/// Query parameters for popular coffees endpoints
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PopularCoffeesQueryParams {
    /// Maximum number of results to return
    pub limit: Option<i32>,
    /// Start date for the period (ISO 8601 format)
    pub start_date: Option<DateTime<Utc>>,
    /// End date for the period (ISO 8601 format)
    pub end_date: Option<DateTime<Utc>>,
}

/// Query parameters for trending coffees endpoint
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendingQueryParams {
    /// Start date for current period (ISO 8601 format)
    pub current_start: Option<DateTime<Utc>>,
    /// End date for current period (ISO 8601 format)
    pub current_end: Option<DateTime<Utc>>,
    /// Start date for previous period (ISO 8601 format)
    pub previous_start: Option<DateTime<Utc>>,
    /// End date for previous period (ISO 8601 format)
    pub previous_end: Option<DateTime<Utc>>,
    /// Maximum number of results to return
    pub limit: Option<i32>,
}

/// Popular coffees controller
pub struct PopularCoffeesController {
    popular_service: Arc<PopularCoffeesService>,
    trend_service: Arc<TrendCalculationService>,
}

impl PopularCoffeesController {
    /// Create a new popular coffees controller
    pub fn new(
        popular_service: Arc<PopularCoffeesService>,
        trend_service: Arc<TrendCalculationService>,
    ) -> Self {
        Self {
            popular_service,
            trend_service,
        }
    }

    /// GET /api/v1/admin/analytics/coffees/most-ordered
    /// Returns the most ordered coffees ranked by order count
    pub async fn get_most_ordered(
        State(controller): State<Arc<Self>>,
        Query(params): Query<PopularCoffeesQueryParams>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
        let start_time = std::time::Instant::now();

        // Parse and validate date range
        let date_range = TimePeriodFilter::parse_date_range(params.start_date, params.end_date)
            .map_err(|e| {
                let metadata = ResponseMetadata::new(serde_json::json!({
                    "startDate": params.start_date,
                    "endDate": params.end_date,
                    "limit": params.limit,
                }));
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error(e, metadata)),
                )
            })?;

        // Validate limit parameter
        let limit = params.limit.unwrap_or(10);
        if limit <= 0 {
            let metadata = ResponseMetadata::new(serde_json::json!({
                "startDate": params.start_date,
                "endDate": params.end_date,
                "limit": params.limit,
            }));
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(
                    "Limit must be a positive integer".to_string(),
                    metadata,
                )),
            ));
        }

        // Get most ordered coffees
        let coffees = controller
            .popular_service
            .get_most_ordered(date_range, limit)
            .await
            .map_err(|e| {
                let metadata = ResponseMetadata::new(serde_json::json!({
                    "startDate": params.start_date,
                    "endDate": params.end_date,
                    "limit": params.limit,
                }));
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(
                        format!("Failed to get most ordered coffees: {}", e),
                        metadata,
                    )),
                )
            })?;

        // Build response metadata
        let execution_time = start_time.elapsed().as_millis() as u64;
        let metadata = ResponseMetadata::new(serde_json::json!({
            "startDate": params.start_date,
            "endDate": params.end_date,
            "limit": limit,
        }))
        .with_result_count(coffees.len())
        .with_execution_time(execution_time);

        Ok((
            StatusCode::OK,
            Json(ApiResponse::success(coffees, metadata)),
        ))
    }

    /// GET /api/v1/admin/analytics/coffees/highest-rated
    /// Returns the highest rated coffees ranked by average rating
    pub async fn get_highest_rated(
        State(controller): State<Arc<Self>>,
        Query(params): Query<PopularCoffeesQueryParams>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
        let start_time = std::time::Instant::now();

        // Validate limit parameter
        let limit = params.limit.unwrap_or(10);
        if limit <= 0 {
            let metadata = ResponseMetadata::new(serde_json::json!({
                "limit": params.limit,
            }));
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(
                    "Limit must be a positive integer".to_string(),
                    metadata,
                )),
            ));
        }

        // Get highest rated coffees (with minimum 5 reviews for statistical significance)
        let min_reviews = 5;
        let coffees = controller
            .popular_service
            .get_highest_rated(limit, min_reviews)
            .await
            .map_err(|e| {
                let metadata = ResponseMetadata::new(serde_json::json!({
                    "limit": params.limit,
                    "minReviews": min_reviews,
                }));
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(
                        format!("Failed to get highest rated coffees: {}", e),
                        metadata,
                    )),
                )
            })?;

        // Build response metadata
        let execution_time = start_time.elapsed().as_millis() as u64;
        let metadata = ResponseMetadata::new(serde_json::json!({
            "limit": limit,
            "minReviews": min_reviews,
        }))
        .with_result_count(coffees.len())
        .with_execution_time(execution_time);

        Ok((
            StatusCode::OK,
            Json(ApiResponse::success(coffees, metadata)),
        ))
    }

    /// GET /api/v1/admin/analytics/coffees/trending
    /// Returns trending coffees by comparing two time periods
    pub async fn get_trending(
        State(controller): State<Arc<Self>>,
        Query(params): Query<TrendingQueryParams>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
        let start_time = std::time::Instant::now();

        // Parse and validate current period
        let current_range = TimePeriodFilter::parse_date_range(
            params.current_start,
            params.current_end,
        )
        .map_err(|e| {
            let metadata = ResponseMetadata::new(serde_json::json!({
                "currentStart": params.current_start,
                "currentEnd": params.current_end,
                "previousStart": params.previous_start,
                "previousEnd": params.previous_end,
                "limit": params.limit,
            }));
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(
                    format!("Invalid current period: {}", e),
                    metadata,
                )),
            )
        })?;

        // Parse and validate previous period
        let previous_range = TimePeriodFilter::parse_date_range(
            params.previous_start,
            params.previous_end,
        )
        .map_err(|e| {
            let metadata = ResponseMetadata::new(serde_json::json!({
                "currentStart": params.current_start,
                "currentEnd": params.current_end,
                "previousStart": params.previous_start,
                "previousEnd": params.previous_end,
                "limit": params.limit,
            }));
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(
                    format!("Invalid previous period: {}", e),
                    metadata,
                )),
            )
        })?;

        // Validate limit parameter
        let limit = params.limit.unwrap_or(10);
        if limit <= 0 {
            let metadata = ResponseMetadata::new(serde_json::json!({
                "currentStart": params.current_start,
                "currentEnd": params.current_end,
                "previousStart": params.previous_start,
                "previousEnd": params.previous_end,
                "limit": params.limit,
            }));
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(
                    "Limit must be a positive integer".to_string(),
                    metadata,
                )),
            ));
        }

        // Calculate trending coffees
        let trending = controller
            .trend_service
            .calculate_trending_items(current_range, previous_range, limit)
            .await
            .map_err(|e| {
                let metadata = ResponseMetadata::new(serde_json::json!({
                    "currentStart": params.current_start,
                    "currentEnd": params.current_end,
                    "previousStart": params.previous_start,
                    "previousEnd": params.previous_end,
                    "limit": params.limit,
                }));
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(
                        format!("Failed to calculate trending coffees: {}", e),
                        metadata,
                    )),
                )
            })?;

        // Build response metadata
        let execution_time = start_time.elapsed().as_millis() as u64;
        let metadata = ResponseMetadata::new(serde_json::json!({
            "currentStart": params.current_start,
            "currentEnd": params.current_end,
            "previousStart": params.previous_start,
            "previousEnd": params.previous_end,
            "limit": limit,
        }))
        .with_result_count(trending.len())
        .with_execution_time(execution_time);

        Ok((
            StatusCode::OK,
            Json(ApiResponse::success(trending, metadata)),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::repositories::{OrdersAnalyticsRepository, ReviewsAnalyticsRepository};
    use sqlx::PgPool;

    fn create_test_controller() -> PopularCoffeesController {
        let pool = PgPool::connect_lazy("postgresql://test").unwrap();
        let orders_repo = OrdersAnalyticsRepository::new(pool.clone());
        let reviews_repo = ReviewsAnalyticsRepository::new(pool);
        
        let popular_service = Arc::new(PopularCoffeesService::new(
            orders_repo.clone(),
            reviews_repo,
        ));
        let trend_service = Arc::new(TrendCalculationService::new(orders_repo));
        
        PopularCoffeesController::new(popular_service, trend_service)
    }

    #[test]
    fn test_controller_creation() {
        let controller = create_test_controller();
        assert!(std::mem::size_of_val(&controller) > 0);
    }

    #[test]
    fn test_popular_coffees_query_params_deserialization() {
        let json = r#"{"limit":5,"startDate":"2024-01-01T00:00:00Z","endDate":"2024-01-31T23:59:59Z"}"#;
        let params: PopularCoffeesQueryParams = serde_json::from_str(json).unwrap();
        
        assert_eq!(params.limit, Some(5));
        assert!(params.start_date.is_some());
        assert!(params.end_date.is_some());
    }

    #[test]
    fn test_trending_query_params_deserialization() {
        let json = r#"{
            "currentStart":"2024-02-01T00:00:00Z",
            "currentEnd":"2024-02-29T23:59:59Z",
            "previousStart":"2024-01-01T00:00:00Z",
            "previousEnd":"2024-01-31T23:59:59Z",
            "limit":10
        }"#;
        let params: TrendingQueryParams = serde_json::from_str(json).unwrap();
        
        assert!(params.current_start.is_some());
        assert!(params.current_end.is_some());
        assert!(params.previous_start.is_some());
        assert!(params.previous_end.is_some());
        assert_eq!(params.limit, Some(10));
    }

    #[test]
    fn test_query_params_optional_fields() {
        let json = r#"{}"#;
        let params: PopularCoffeesQueryParams = serde_json::from_str(json).unwrap();
        
        assert!(params.limit.is_none());
        assert!(params.start_date.is_none());
        assert!(params.end_date.is_none());
    }

    #[test]
    fn test_default_limit() {
        let json = r#"{}"#;
        let params: PopularCoffeesQueryParams = serde_json::from_str(json).unwrap();
        let limit = params.limit.unwrap_or(10);
        assert_eq!(limit, 10);
    }
}
