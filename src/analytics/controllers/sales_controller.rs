// Sales statistics API controller
// Handles endpoints for sales totals, aggregations, and trends

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
    services::SalesAggregationService,
    types::{ApiResponse, ResponseMetadata, SalesStatistics, SalesByPeriod, SalesTrend, TimePeriod},
    utils::TimePeriodFilter,
};

/// Query parameters for sales endpoints
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalesQueryParams {
    /// Start date for the period (ISO 8601 format)
    pub start_date: Option<DateTime<Utc>>,
    /// End date for the period (ISO 8601 format)
    pub end_date: Option<DateTime<Utc>>,
    /// Granularity for aggregation (daily, weekly, monthly, custom)
    pub period: Option<String>,
}

/// Sales statistics controller
pub struct SalesStatisticsController {
    service: Arc<SalesAggregationService>,
}

impl SalesStatisticsController {
    /// Create a new sales statistics controller
    pub fn new(service: Arc<SalesAggregationService>) -> Self {
        Self { service }
    }

    /// GET /api/v1/admin/analytics/sales/total
    /// Returns total sales count for the specified period
    pub async fn get_total_sales(
        State(controller): State<Arc<Self>>,
        Query(params): Query<SalesQueryParams>,
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

        // Calculate total sales
        let sales = controller
            .service
            .calculate_total_sales(date_range)
            .await
            .map_err(|e| {
                let metadata = ResponseMetadata::new(serde_json::json!({
                    "startDate": params.start_date,
                    "endDate": params.end_date,
                }));
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(
                        format!("Failed to calculate sales: {}", e),
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
        .with_execution_time(execution_time);

        Ok((
            StatusCode::OK,
            Json(ApiResponse::success(sales, metadata)),
        ))
    }

    /// GET /api/v1/admin/analytics/sales/by-period
    /// Returns sales aggregated by time period (daily, weekly, monthly)
    pub async fn get_sales_by_period(
        State(controller): State<Arc<Self>>,
        Query(params): Query<SalesQueryParams>,
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

        // Aggregate sales by period
        let sales_by_period = controller
            .service
            .aggregate_sales_by_period(date_range, period)
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
                        format!("Failed to aggregate sales: {}", e),
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
        .with_result_count(sales_by_period.len())
        .with_execution_time(execution_time);

        Ok((
            StatusCode::OK,
            Json(ApiResponse::success(sales_by_period, metadata)),
        ))
    }

    /// GET /api/v1/admin/analytics/sales/trends
    /// Returns sales trends as time-series data
    pub async fn get_sales_trends(
        State(controller): State<Arc<Self>>,
        Query(params): Query<SalesQueryParams>,
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

        // Calculate sales trends
        let trends = controller
            .service
            .calculate_trends(date_range)
            .await
            .map_err(|e| {
                let metadata = ResponseMetadata::new(serde_json::json!({
                    "startDate": params.start_date,
                    "endDate": params.end_date,
                }));
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(
                        format!("Failed to calculate trends: {}", e),
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
    use crate::analytics::repositories::OrdersAnalyticsRepository;
    use sqlx::PgPool;

    fn create_test_controller() -> SalesStatisticsController {
        let pool = PgPool::connect_lazy("postgresql://test").unwrap();
        let orders_repo = OrdersAnalyticsRepository::new(pool);
        let service = Arc::new(SalesAggregationService::new(orders_repo));
        SalesStatisticsController::new(service)
    }

    #[test]
    fn test_controller_creation() {
        let controller = create_test_controller();
        // Controller should be created successfully
        assert!(std::mem::size_of_val(&controller) > 0);
    }

    #[test]
    fn test_sales_query_params_deserialization() {
        let json = r#"{"startDate":"2024-01-01T00:00:00Z","endDate":"2024-01-31T23:59:59Z","period":"daily"}"#;
        let params: SalesQueryParams = serde_json::from_str(json).unwrap();
        
        assert!(params.start_date.is_some());
        assert!(params.end_date.is_some());
        assert_eq!(params.period, Some("daily".to_string()));
    }

    #[test]
    fn test_sales_query_params_optional_fields() {
        let json = r#"{}"#;
        let params: SalesQueryParams = serde_json::from_str(json).unwrap();
        
        assert!(params.start_date.is_none());
        assert!(params.end_date.is_none());
        assert!(params.period.is_none());
    }
}
