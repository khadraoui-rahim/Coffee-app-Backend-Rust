use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Time period granularity for analytics aggregation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TimePeriod {
    Daily,
    Weekly,
    Monthly,
    Custom,
}

/// Date range filter for analytics queries
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DateRange {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

impl DateRange {
    /// Validates that start_date is before end_date
    pub fn validate(&self) -> Result<(), String> {
        if self.start_date >= self.end_date {
            return Err("start_date must be before end_date".to_string());
        }
        Ok(())
    }
}

/// Sales statistics data
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SalesStatistics {
    pub total_sales: i64,
    pub period: Option<TimePeriod>,
    pub date_range: Option<DateRange>,
}

/// Sales data aggregated by time period
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SalesByPeriod {
    pub period: String,
    pub sales_count: i64,
    pub timestamp: DateTime<Utc>,
}

/// Sales trend data point
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SalesTrend {
    pub timestamp: DateTime<Utc>,
    pub value: i64,
}

/// Popular coffee item with order statistics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PopularCoffee {
    pub coffee_id: i32,
    pub coffee_name: String,
    pub order_count: i64,
    pub average_rating: Option<Decimal>,
    pub trend_percentage: Option<Decimal>,
}

/// Revenue data aggregated by time period
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RevenueByPeriod {
    pub period: String,
    pub revenue: Decimal,
    pub timestamp: DateTime<Utc>,
}

/// Revenue data aggregated by coffee type
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RevenueByCoffee {
    pub coffee_id: i32,
    pub coffee_name: String,
    pub revenue: Decimal,
}

/// Rating statistics data
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RatingStatistics {
    pub average_rating: Decimal,
    pub total_reviews: i64,
    pub coffee_id: Option<i32>,
}

/// Rating distribution by star value
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RatingDistribution {
    pub rating: i32,
    pub count: i64,
}

/// Rating trend data point
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RatingTrend {
    pub timestamp: DateTime<Utc>,
    pub average_rating: Decimal,
}

/// Generic API response envelope
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub metadata: ResponseMetadata,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, metadata: ResponseMetadata) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata,
        }
    }

    pub fn error(error: String, metadata: ResponseMetadata) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            metadata,
        }
    }
}

/// Response metadata for analytics queries
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMetadata {
    pub timestamp: DateTime<Utc>,
    pub query_params: serde_json::Value,
    pub result_count: Option<usize>,
    pub execution_time_ms: Option<u64>,
}

impl ResponseMetadata {
    pub fn new(query_params: serde_json::Value) -> Self {
        Self {
            timestamp: Utc::now(),
            query_params,
            result_count: None,
            execution_time_ms: None,
        }
    }

    pub fn with_result_count(mut self, count: usize) -> Self {
        self.result_count = Some(count);
        self
    }

    pub fn with_execution_time(mut self, time_ms: u64) -> Self {
        self.execution_time_ms = Some(time_ms);
        self
    }
}

/// Query parameters for analytics endpoints
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsQueryParams {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub period: Option<TimePeriod>,
    pub limit: Option<i32>,
    pub coffee_id: Option<i32>,
}

impl Default for AnalyticsQueryParams {
    fn default() -> Self {
        Self {
            start_date: None,
            end_date: None,
            period: None,
            limit: Some(10),
            coffee_id: None,
        }
    }
}

/// Order status enum (must match database enum)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Preparing,
    Ready,
    Completed,
    Cancelled,
}

impl OrderStatus {
    pub fn is_completed(&self) -> bool {
        matches!(self, OrderStatus::Completed)
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::Pending => "pending",
            OrderStatus::Confirmed => "confirmed",
            OrderStatus::Preparing => "preparing",
            OrderStatus::Ready => "ready",
            OrderStatus::Completed => "completed",
            OrderStatus::Cancelled => "cancelled",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_range_validation() {
        let valid_range = DateRange {
            start_date: Utc::now() - chrono::Duration::days(7),
            end_date: Utc::now(),
        };
        assert!(valid_range.validate().is_ok());

        let invalid_range = DateRange {
            start_date: Utc::now(),
            end_date: Utc::now() - chrono::Duration::days(7),
        };
        assert!(invalid_range.validate().is_err());
    }

    #[test]
    fn test_order_status_is_completed() {
        assert!(OrderStatus::Completed.is_completed());
        assert!(!OrderStatus::Pending.is_completed());
        assert!(!OrderStatus::Cancelled.is_completed());
    }

    #[test]
    fn test_api_response_success() {
        let metadata = ResponseMetadata::new(serde_json::json!({}));
        let response = ApiResponse::success("test data", metadata);
        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let metadata = ResponseMetadata::new(serde_json::json!({}));
        let response: ApiResponse<String> = ApiResponse::error("test error".to_string(), metadata);
        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
    }
}
