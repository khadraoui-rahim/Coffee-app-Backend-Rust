// Rating insights API controller
// Handles endpoints for average ratings, rating distribution, and rating trends

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
    services::RatingAnalysisService,
    types::{ApiResponse, ResponseMetadata, RatingStatistics, RatingDistribution, RatingTrend},
    utils::TimePeriodFilter,
};

/// Query parameters for rating endpoints
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingQueryParams {
    /// Start date for the period (ISO 8601 format)
    pub start_date: Option<DateTime<Utc>>,
    /// End date for the period (ISO 8601 format)
    pub end_date: Option<DateTime<Utc>>,
    /// Optional coffee ID to filter ratings for a specific coffee
    pub coffee_id: Option<i32>,
}

/// Rating insights controller
pub struct RatingInsightsController {
    service: Arc<RatingAnalysisService>,
}

impl RatingInsightsController {
    /// Create a new rating insights controller
    pub fn new(service: Arc<RatingAnalysisService>) -> Self {
        Self { service }
    }

    /// GET /api/v1/admin/analytics/ratings/average
    /// Returns average rating with optional coffee filter
    pub async fn get_average_rating(
        State(controller): State<Arc<Self>>,
        Query(params): Query<RatingQueryParams>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
        let start_time = std::time::Instant::now();

        // Calculate average rating (no date range filtering for this endpoint)
        let rating_stats = controller
            .service
            .calculate_average_rating(params.coffee_id)
            .await
            .map_err(|e| {
                let metadata = ResponseMetadata::new(serde_json::json!({
                    "coffeeId": params.coffee_id,
                }));
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(
                        format!("Failed to calculate average rating: {}", e),
                        metadata,
                    )),
                )
            })?;

        // Build response metadata
        let execution_time = start_time.elapsed().as_millis() as u64;
        let metadata = ResponseMetadata::new(serde_json::json!({
            "coffeeId": params.coffee_id,
        }))
        .with_execution_time(execution_time);

        Ok((
            StatusCode::OK,
            Json(ApiResponse::success(rating_stats, metadata)),
        ))
    }

    /// GET /api/v1/admin/analytics/ratings/distribution
    /// Returns rating distribution grouped by rating value (1-5 stars)
    pub async fn get_rating_distribution(
        State(controller): State<Arc<Self>>,
        Query(params): Query<RatingQueryParams>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
        let start_time = std::time::Instant::now();

        // Analyze rating distribution (no date range filtering for this endpoint)
        let distribution = controller
            .service
            .analyze_rating_distribution(params.coffee_id)
            .await
            .map_err(|e| {
                let metadata = ResponseMetadata::new(serde_json::json!({
                    "coffeeId": params.coffee_id,
                }));
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(
                        format!("Failed to analyze rating distribution: {}", e),
                        metadata,
                    )),
                )
            })?;

        // Build response metadata
        let execution_time = start_time.elapsed().as_millis() as u64;
        let metadata = ResponseMetadata::new(serde_json::json!({
            "coffeeId": params.coffee_id,
        }))
        .with_result_count(distribution.len())
        .with_execution_time(execution_time);

        Ok((
            StatusCode::OK,
            Json(ApiResponse::success(distribution, metadata)),
        ))
    }

    /// GET /api/v1/admin/analytics/ratings/trends
    /// Returns rating trends as time-series data
    pub async fn get_rating_trends(
        State(controller): State<Arc<Self>>,
        Query(params): Query<RatingQueryParams>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
        let start_time = std::time::Instant::now();

        // Parse and validate date range
        let date_range = TimePeriodFilter::parse_date_range(params.start_date, params.end_date)
            .map_err(|e| {
                let metadata = ResponseMetadata::new(serde_json::json!({
                    "startDate": params.start_date,
                    "endDate": params.end_date,
                    "coffeeId": params.coffee_id,
                }));
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error(e, metadata)),
                )
            })?;

        // Calculate rating trends
        let trends = controller
            .service
            .analyze_trends(date_range, params.coffee_id)
            .await
            .map_err(|e| {
                let metadata = ResponseMetadata::new(serde_json::json!({
                    "startDate": params.start_date,
                    "endDate": params.end_date,
                    "coffeeId": params.coffee_id,
                }));
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(
                        format!("Failed to calculate rating trends: {}", e),
                        metadata,
                    )),
                )
            })?;

        // Build response metadata
        let execution_time = start_time.elapsed().as_millis() as u64;
        let metadata = ResponseMetadata::new(serde_json::json!({
            "startDate": params.start_date,
            "endDate": params.end_date,
            "coffeeId": params.coffee_id,
        }))
        .with_result_count(trends.len())
        .with_execution_time(execution_time);

        Ok((
            StatusCode::OK,
            Json(ApiResponse::success(trends, metadata)),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::repositories::ReviewsAnalyticsRepository;
    use sqlx::PgPool;

    fn create_test_controller() -> RatingInsightsController {
        let pool = PgPool::connect_lazy("postgresql://test").unwrap();
        let reviews_repo = ReviewsAnalyticsRepository::new(pool);
        let service = Arc::new(RatingAnalysisService::new(reviews_repo));
        RatingInsightsController::new(service)
    }

    #[test]
    fn test_controller_creation() {
        let controller = create_test_controller();
        // Controller should be created successfully
        assert!(std::mem::size_of_val(&controller) > 0);
    }

    #[test]
    fn test_rating_query_params_deserialization() {
        let json = r#"{"startDate":"2024-01-01T00:00:00Z","endDate":"2024-01-31T23:59:59Z","coffeeId":1}"#;
        let params: RatingQueryParams = serde_json::from_str(json).unwrap();
        
        assert!(params.start_date.is_some());
        assert!(params.end_date.is_some());
        assert_eq!(params.coffee_id, Some(1));
    }

    #[test]
    fn test_rating_query_params_optional_fields() {
        let json = r#"{}"#;
        let params: RatingQueryParams = serde_json::from_str(json).unwrap();
        
        assert!(params.start_date.is_none());
        assert!(params.end_date.is_none());
        assert!(params.coffee_id.is_none());
    }

    #[test]
    fn test_rating_query_params_camel_case() {
        // Test that camelCase deserialization works
        let json = r#"{"startDate":"2024-01-01T00:00:00Z","endDate":"2024-01-31T23:59:59Z","coffeeId":5}"#;
        let result: Result<RatingQueryParams, _> = serde_json::from_str(json);
        assert!(result.is_ok());
        
        let params = result.unwrap();
        assert_eq!(params.coffee_id, Some(5));
    }

    #[test]
    fn test_rating_query_params_without_coffee_id() {
        // Test query params without coffee_id (for all coffees)
        let json = r#"{"startDate":"2024-01-01T00:00:00Z","endDate":"2024-01-31T23:59:59Z"}"#;
        let params: RatingQueryParams = serde_json::from_str(json).unwrap();
        
        assert!(params.start_date.is_some());
        assert!(params.end_date.is_some());
        assert!(params.coffee_id.is_none());
    }
}
