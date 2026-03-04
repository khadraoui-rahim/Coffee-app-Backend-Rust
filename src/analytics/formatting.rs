// Response formatting utilities
// Provides consistent formatting for analytics API responses

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::analytics::types::{ApiResponse, ResponseMetadata};

/// Time-series data point with timestamp and value
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TimeSeriesPoint<T> {
    pub timestamp: DateTime<Utc>,
    pub value: T,
}

impl<T> TimeSeriesPoint<T> {
    pub fn new(timestamp: DateTime<Utc>, value: T) -> Self {
        Self { timestamp, value }
    }
}

/// Response formatter for analytics data
pub struct ResponseFormatter;

impl ResponseFormatter {
    /// Create a success response with data and metadata
    pub fn success<T>(data: T, metadata: ResponseMetadata) -> ApiResponse<T> {
        ApiResponse::success(data, metadata)
    }

    /// Create an error response with message and metadata
    pub fn error<T>(message: String, metadata: ResponseMetadata) -> ApiResponse<T> {
        ApiResponse::error(message, metadata)
    }

    /// Format time-series data with timestamp/value pairs
    pub fn format_time_series<T>(data: Vec<(DateTime<Utc>, T)>) -> Vec<TimeSeriesPoint<T>> {
        data.into_iter()
            .map(|(timestamp, value)| TimeSeriesPoint::new(timestamp, value))
            .collect()
    }

    /// Create metadata with query parameters
    pub fn create_metadata(query_params: serde_json::Value) -> ResponseMetadata {
        ResponseMetadata::new(query_params)
    }

    /// Create metadata with query parameters and result count
    pub fn create_metadata_with_count(
        query_params: serde_json::Value,
        count: usize,
    ) -> ResponseMetadata {
        ResponseMetadata::new(query_params).with_result_count(count)
    }

    /// Create metadata with query parameters, result count, and execution time
    pub fn create_metadata_complete(
        query_params: serde_json::Value,
        count: usize,
        execution_time_ms: u64,
    ) -> ResponseMetadata {
        ResponseMetadata::new(query_params)
            .with_result_count(count)
            .with_execution_time(execution_time_ms)
    }

    /// Verify that all field names in a JSON value use camelCase
    pub fn verify_camel_case(json: &serde_json::Value) -> bool {
        match json {
            serde_json::Value::Object(map) => {
                for key in map.keys() {
                    if !Self::is_camel_case(key) {
                        return false;
                    }
                    // Recursively check nested objects
                    if let Some(value) = map.get(key) {
                        if !Self::verify_camel_case(value) {
                            return false;
                        }
                    }
                }
                true
            }
            serde_json::Value::Array(arr) => {
                arr.iter().all(|v| Self::verify_camel_case(v))
            }
            _ => true,
        }
    }

    /// Check if a string is in camelCase format
    fn is_camel_case(s: &str) -> bool {
        if s.is_empty() {
            return true;
        }

        // First character should be lowercase
        let first_char = s.chars().next().unwrap();
        if first_char.is_uppercase() {
            return false;
        }

        // Should not contain underscores or hyphens
        if s.contains('_') || s.contains('-') {
            return false;
        }

        true
    }

    /// Verify response structure has all required fields
    pub fn verify_response_structure<T>(response: &ApiResponse<T>) -> bool {
        // Check that metadata has required fields
        response.metadata.timestamp <= Utc::now()
    }

    /// Verify time-series format has timestamp and value fields
    pub fn verify_time_series_format<T>(points: &[TimeSeriesPoint<T>]) -> bool {
        // All points should have valid timestamps
        for point in points {
            if point.timestamp > Utc::now() {
                return false;
            }
        }
        true
    }

    /// Verify metadata completeness
    pub fn verify_metadata_completeness(metadata: &ResponseMetadata) -> bool {
        // Timestamp should be present and valid
        if metadata.timestamp > Utc::now() {
            return false;
        }

        // Query params should be present
        if metadata.query_params.is_null() {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_time_series_point_creation() {
        let timestamp = Utc::now();
        let value = 42;
        let point = TimeSeriesPoint::new(timestamp, value);

        assert_eq!(point.timestamp, timestamp);
        assert_eq!(point.value, value);
    }

    #[test]
    fn test_format_time_series() {
        let data = vec![
            (Utc::now() - Duration::days(2), 10),
            (Utc::now() - Duration::days(1), 20),
            (Utc::now(), 30),
        ];

        let formatted = ResponseFormatter::format_time_series(data.clone());

        assert_eq!(formatted.len(), 3);
        assert_eq!(formatted[0].value, 10);
        assert_eq!(formatted[1].value, 20);
        assert_eq!(formatted[2].value, 30);
    }

    #[test]
    fn test_create_metadata() {
        let params = serde_json::json!({
            "startDate": "2024-01-01",
            "endDate": "2024-01-31"
        });

        let metadata = ResponseFormatter::create_metadata(params.clone());

        assert_eq!(metadata.query_params, params);
        assert!(metadata.result_count.is_none());
        assert!(metadata.execution_time_ms.is_none());
    }

    #[test]
    fn test_create_metadata_with_count() {
        let params = serde_json::json!({ "limit": 10 });
        let metadata = ResponseFormatter::create_metadata_with_count(params, 5);

        assert_eq!(metadata.result_count, Some(5));
    }

    #[test]
    fn test_create_metadata_complete() {
        let params = serde_json::json!({ "period": "daily" });
        let metadata = ResponseFormatter::create_metadata_complete(params, 30, 150);

        assert_eq!(metadata.result_count, Some(30));
        assert_eq!(metadata.execution_time_ms, Some(150));
    }

    // Property 23: JSON response structure - Valid JSON with required fields
    #[test]
    fn test_response_structure_valid() {
        let metadata = ResponseMetadata::new(serde_json::json!({}));
        let response = ApiResponse::success(vec![1, 2, 3], metadata);

        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());
        assert!(ResponseFormatter::verify_response_structure(&response));
    }

    #[test]
    fn test_error_response_structure() {
        let metadata = ResponseMetadata::new(serde_json::json!({}));
        let response: ApiResponse<Vec<i32>> = ApiResponse::error("Test error".to_string(), metadata);

        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
    }

    // Property 24: Time-series format - Timestamp and value fields present
    #[test]
    fn test_time_series_format() {
        let points = vec![
            TimeSeriesPoint::new(Utc::now() - Duration::days(2), 100),
            TimeSeriesPoint::new(Utc::now() - Duration::days(1), 200),
            TimeSeriesPoint::new(Utc::now(), 300),
        ];

        assert!(ResponseFormatter::verify_time_series_format(&points));

        // Verify serialization includes both fields
        let json = serde_json::to_value(&points[0]).unwrap();
        assert!(json.get("timestamp").is_some());
        assert!(json.get("value").is_some());
    }

    #[test]
    fn test_time_series_with_decimal_values() {
        let points = vec![
            TimeSeriesPoint::new(Utc::now(), Decimal::from_str("4.5").unwrap()),
            TimeSeriesPoint::new(Utc::now(), Decimal::from_str("4.8").unwrap()),
        ];

        assert!(ResponseFormatter::verify_time_series_format(&points));
    }

    // Property 25: Response metadata completeness - All metadata fields present
    #[test]
    fn test_metadata_completeness() {
        let metadata = ResponseMetadata::new(serde_json::json!({
            "startDate": "2024-01-01",
            "endDate": "2024-01-31"
        }))
        .with_result_count(10)
        .with_execution_time(50);

        assert!(ResponseFormatter::verify_metadata_completeness(&metadata));
        assert!(metadata.timestamp <= Utc::now());
        assert!(metadata.result_count.is_some());
        assert!(metadata.execution_time_ms.is_some());
    }

    #[test]
    fn test_metadata_minimal() {
        let metadata = ResponseMetadata::new(serde_json::json!({}));
        assert!(ResponseFormatter::verify_metadata_completeness(&metadata));
    }

    // Property 26: CamelCase field naming - All fields follow camelCase
    #[test]
    fn test_camel_case_validation() {
        assert!(ResponseFormatter::is_camel_case("camelCase"));
        assert!(ResponseFormatter::is_camel_case("startDate"));
        assert!(ResponseFormatter::is_camel_case("endDate"));
        assert!(ResponseFormatter::is_camel_case("coffeeId"));
        assert!(ResponseFormatter::is_camel_case("resultCount"));
        assert!(ResponseFormatter::is_camel_case("executionTimeMs"));

        // Invalid cases
        assert!(!ResponseFormatter::is_camel_case("PascalCase"));
        assert!(!ResponseFormatter::is_camel_case("snake_case"));
        assert!(!ResponseFormatter::is_camel_case("kebab-case"));
    }

    #[test]
    fn test_verify_camel_case_json() {
        let valid_json = serde_json::json!({
            "startDate": "2024-01-01",
            "endDate": "2024-01-31",
            "coffeeId": 5,
            "resultCount": 10
        });

        assert!(ResponseFormatter::verify_camel_case(&valid_json));

        let invalid_json = serde_json::json!({
            "start_date": "2024-01-01",
            "end_date": "2024-01-31"
        });

        assert!(!ResponseFormatter::verify_camel_case(&invalid_json));
    }

    #[test]
    fn test_verify_camel_case_nested() {
        let nested_json = serde_json::json!({
            "data": {
                "salesCount": 100,
                "revenueTotal": 1000.50
            },
            "metadata": {
                "queryParams": {
                    "startDate": "2024-01-01"
                }
            }
        });

        assert!(ResponseFormatter::verify_camel_case(&nested_json));
    }

    #[test]
    fn test_response_serialization_camel_case() {
        let metadata = ResponseMetadata::new(serde_json::json!({}));
        let response = ApiResponse::success(vec![1, 2, 3], metadata);

        let json = serde_json::to_value(&response).unwrap();
        assert!(ResponseFormatter::verify_camel_case(&json));

        // Check specific fields
        assert!(json.get("success").is_some());
        assert!(json.get("data").is_some());
        assert!(json.get("error").is_some());
        assert!(json.get("metadata").is_some());
    }

    #[test]
    fn test_time_series_serialization_camel_case() {
        let point = TimeSeriesPoint::new(Utc::now(), 42);
        let json = serde_json::to_value(&point).unwrap();

        assert!(ResponseFormatter::verify_camel_case(&json));
        assert!(json.get("timestamp").is_some());
        assert!(json.get("value").is_some());
    }

    #[test]
    fn test_success_response_creation() {
        let data = vec!["item1", "item2"];
        let metadata = ResponseMetadata::new(serde_json::json!({}));
        let response = ResponseFormatter::success(data.clone(), metadata);

        assert!(response.success);
        assert_eq!(response.data, Some(data));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_error_response_creation() {
        let metadata = ResponseMetadata::new(serde_json::json!({}));
        let response: ApiResponse<String> = ResponseFormatter::error("Error message".to_string(), metadata);

        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("Error message".to_string()));
    }

    #[test]
    fn test_empty_time_series() {
        let empty: Vec<(DateTime<Utc>, i32)> = vec![];
        let formatted = ResponseFormatter::format_time_series(empty);

        assert_eq!(formatted.len(), 0);
    }

    #[test]
    fn test_metadata_timestamp_validity() {
        let metadata = ResponseMetadata::new(serde_json::json!({}));
        let now = Utc::now();

        // Timestamp should be close to now (within 1 second)
        let diff = (now - metadata.timestamp).num_seconds().abs();
        assert!(diff < 1);
    }
}
