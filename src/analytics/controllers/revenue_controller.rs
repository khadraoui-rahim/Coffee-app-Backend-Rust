// Revenue reports API controller
// Handles endpoints for revenue aggregation by period and by coffee

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
    services::RevenueCalculationService,
    types::{ApiResponse, ResponseMetadata, RevenueByPeriod, RevenueByCoffee, TimePeriod},
    utils::TimePeriodFilter,
};

/// Query parameters for revenue endpoints
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevenueQueryParams {
    /// Start date for the period (ISO 8601 format)
    pub start_date: Option<DateTime<Utc>>,
    /// End date for the period (ISO 8601 format)
    pub end_date: Option<DateTime<Utc>>,
    /// Granularity for aggregation (daily, weekly, monthly, custom)
    pub period: Option<String>,
}

/// Revenue reports controller
pub struct RevenueReportsController {
    service: Arc<RevenueCalculationService>,
}

impl RevenueReportsController {
    /// Create a new revenue reports controller
    pub fn new(service: Arc<RevenueCalculationService>) -> Self {
        Self { service }
    }

    /// GET /api/v1/admin/analytics/revenue/by-period
    /// Returns revenue aggregated by time period (daily, weekly, monthly)
    /// All monetary values have exactly 2 decimal places
    pub async fn get_revenue_by_period(
        State(controller): State<Arc<Self>>,
        Query(params): Query<RevenueQueryParams>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
        let start_time = std::time::Instant::now();

        // Parse and validate date range
        let date_range = TimePeriodFilter::parse_date_range(params.start_date, params.end_date)
            .map_err(|e| {
                let metadata = ResponseMetadata::new(serde_json::json!({
                    "startDate": params.start_date,
                    "endDate": params.end_date,
                    "period": params.period,
                }));
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error(e, metadata)),
                )
            })?;

        // Validate and parse period parameter
        let period = if let Some(period_str) = &params.period {
            TimePeriodFilter::validate_period(period_str).map_err(|e| {
                let metadata = ResponseMetadata::new(serde_json::json!({
                    "startDate": params.start_date,
                    "endDate": params.end_date,
                    "period": params.period,
                }));
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error(e, metadata)),
                )
            })?
        } else {
            TimePeriod::Daily // Default to daily
        };

        // Calculate revenue by period
        let revenue_by_period = controller
            .service
            .calculate_revenue_by_period(date_range, period)
            .await
            .map_err(|e| {
                let metadata = ResponseMetadata::new(serde_json::json!({
                    "startDate": params.start_date,
                    "endDate": params.end_date,
                    "period": params.period,
                }));
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(
                        format!("Failed to calculate revenue: {}", e),
                        metadata,
                    )),
                )
            })?;

        // Build response metadata
        let execution_time = start_time.elapsed().as_millis() as u64;
        let metadata = ResponseMetadata::new(serde_json::json!({
            "startDate": params.start_date,
            "endDate": params.end_date,
            "period": params.period,
        }))
        .with_result_count(revenue_by_period.len())
        .with_execution_time(execution_time);

        Ok((
            StatusCode::OK,
            Json(ApiResponse::success(revenue_by_period, metadata)),
        ))
    }

    /// GET /api/v1/admin/analytics/revenue/by-coffee
    /// Returns revenue aggregated by coffee type
    /// All monetary values have exactly 2 decimal places
    pub async fn get_revenue_by_coffee(
        State(controller): State<Arc<Self>>,
        Query(params): Query<RevenueQueryParams>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
        let start_time = std::time::Instant::now();

        // Parse and validate date range
        let date_range = TimePeriodFilter::parse_date_range(params.start_date, params.end_date)
            .map_err(|e| {
                let metadata = ResponseMetadata::new(serde_json::json!({
                    "startDate": params.start_date,
                    "endDate": params.end_date,
                }));
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error(e, metadata)),
                )
            })?;

        // Calculate revenue by coffee
        let revenue_by_coffee = controller
            .service
            .calculate_revenue_by_coffee(date_range)
            .await
            .map_err(|e| {
                let metadata = ResponseMetadata::new(serde_json::json!({
                    "startDate": params.start_date,
                    "endDate": params.end_date,
                }));
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(
                        format!("Failed to calculate revenue by coffee: {}", e),
                        metadata,
                    )),
                )
            })?;

        // Build response metadata
        let execution_time = start_time.elapsed().as_millis() as u64;
        let metadata = ResponseMetadata::new(serde_json::json!({
            "startDate": params.start_date,
            "endDate": params.end_date,
        }))
        .with_result_count(revenue_by_coffee.len())
        .with_execution_time(execution_time);

        Ok((
            StatusCode::OK,
            Json(ApiResponse::success(revenue_by_coffee, metadata)),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::repositories::OrdersAnalyticsRepository;
    use sqlx::PgPool;

    fn create_test_controller() -> RevenueReportsController {
        let pool = PgPool::connect_lazy("postgresql://test").unwrap();
        let orders_repo = OrdersAnalyticsRepository::new(pool);
        let service = Arc::new(RevenueCalculationService::new(orders_repo));
        RevenueReportsController::new(service)
    }

    #[test]
    fn test_controller_creation() {
        let controller = create_test_controller();
        // Controller should be created successfully
        assert!(std::mem::size_of_val(&controller) > 0);
    }

    #[test]
    fn test_revenue_query_params_deserialization() {
        let json = r#"{"startDate":"2024-01-01T00:00:00Z","endDate":"2024-01-31T23:59:59Z","period":"daily"}"#;
        let params: RevenueQueryParams = serde_json::from_str(json).unwrap();
        
        assert!(params.start_date.is_some());
        assert!(params.end_date.is_some());
        assert_eq!(params.period, Some("daily".to_string()));
    }

    #[test]
    fn test_revenue_query_params_optional_fields() {
        let json = r#"{}"#;
        let params: RevenueQueryParams = serde_json::from_str(json).unwrap();
        
        assert!(params.start_date.is_none());
        assert!(params.end_date.is_none());
        assert!(params.period.is_none());
    }

    #[test]
    fn test_revenue_query_params_camel_case() {
        // Test that camelCase deserialization works
        let json = r#"{"startDate":"2024-01-01T00:00:00Z","endDate":"2024-01-31T23:59:59Z"}"#;
        let result: Result<RevenueQueryParams, _> = serde_json::from_str(json);
        assert!(result.is_ok());
    }
}
