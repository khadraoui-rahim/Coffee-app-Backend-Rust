// Unit tests for analytics types and utilities
// This module contains tests for the core analytics data structures

#[cfg(test)]
mod tests {
    use crate::analytics::types::*;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    // ============================================================================
    // DateRange Tests
    // ============================================================================

    #[test]
    fn test_date_range_validation_valid() {
        let valid_range = DateRange {
            start_date: Utc::now() - chrono::Duration::days(7),
            end_date: Utc::now(),
        };
        assert!(valid_range.validate().is_ok());
    }

    #[test]
    fn test_date_range_validation_invalid() {
        let invalid_range = DateRange {
            start_date: Utc::now(),
            end_date: Utc::now() - chrono::Duration::days(7),
        };
        assert!(invalid_range.validate().is_err());
        assert_eq!(
            invalid_range.validate().unwrap_err(),
            "start_date must be before end_date"
        );
    }

    #[test]
    fn test_date_range_validation_equal_dates() {
        let now = Utc::now();
        let equal_range = DateRange {
            start_date: now,
            end_date: now,
        };
        assert!(equal_range.validate().is_err());
    }

    // ============================================================================
    // OrderStatus Tests
    // ============================================================================

    #[test]
    fn test_order_status_is_completed() {
        assert!(OrderStatus::Completed.is_completed());
        assert!(!OrderStatus::Pending.is_completed());
        assert!(!OrderStatus::Confirmed.is_completed());
        assert!(!OrderStatus::Preparing.is_completed());
        assert!(!OrderStatus::Ready.is_completed());
        assert!(!OrderStatus::Cancelled.is_completed());
    }

    // ============================================================================
    // ApiResponse Tests
    // ============================================================================

    #[test]
    fn test_api_response_success() {
        let metadata = ResponseMetadata::new(serde_json::json!({"test": "param"}));
        let response = ApiResponse::success("test data", metadata);
        
        assert!(response.success);
        assert!(response.data.is_some());
        assert_eq!(response.data.unwrap(), "test data");
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let metadata = ResponseMetadata::new(serde_json::json!({"test": "param"}));
        let response: ApiResponse<String> = ApiResponse::error("test error".to_string(), metadata);
        
        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "test error");
    }

    // ============================================================================
    // ResponseMetadata Tests
    // ============================================================================

    #[test]
    fn test_response_metadata_new() {
        let query_params = serde_json::json!({"period": "daily", "limit": 10});
        let metadata = ResponseMetadata::new(query_params.clone());
        
        assert_eq!(metadata.query_params, query_params);
        assert!(metadata.result_count.is_none());
        assert!(metadata.execution_time_ms.is_none());
    }

    #[test]
    fn test_response_metadata_with_result_count() {
        let metadata = ResponseMetadata::new(serde_json::json!({}))
            .with_result_count(42);
        
        assert_eq!(metadata.result_count, Some(42));
    }

    #[test]
    fn test_response_metadata_with_execution_time() {
        let metadata = ResponseMetadata::new(serde_json::json!({}))
            .with_execution_time(150);
        
        assert_eq!(metadata.execution_time_ms, Some(150));
    }

    #[test]
    fn test_response_metadata_chaining() {
        let metadata = ResponseMetadata::new(serde_json::json!({"test": "value"}))
            .with_result_count(100)
            .with_execution_time(250);
        
        assert_eq!(metadata.result_count, Some(100));
        assert_eq!(metadata.execution_time_ms, Some(250));
    }

    // ============================================================================
    // AnalyticsQueryParams Tests
    // ============================================================================

    #[test]
    fn test_analytics_query_params_default() {
        let params = AnalyticsQueryParams::default();
        
        assert!(params.start_date.is_none());
        assert!(params.end_date.is_none());
        assert!(params.period.is_none());
        assert_eq!(params.limit, Some(10));
        assert!(params.coffee_id.is_none());
    }

    // ============================================================================
    // Serialization Tests (camelCase)
    // ============================================================================

    #[test]
    fn test_sales_statistics_serialization() {
        let stats = SalesStatistics {
            total_sales: 100,
            period: Some(TimePeriod::Daily),
            date_range: None,
        };
        
        let json = serde_json::to_value(&stats).unwrap();
        assert!(json.get("totalSales").is_some());
        assert_eq!(json["totalSales"], 100);
    }

    #[test]
    fn test_sales_by_period_serialization() {
        let sales = SalesByPeriod {
            period: "2024-01".to_string(),
            sales_count: 50,
            timestamp: Utc::now(),
        };
        
        let json = serde_json::to_value(&sales).unwrap();
        assert!(json.get("salesCount").is_some());
        assert_eq!(json["salesCount"], 50);
    }

    #[test]
    fn test_popular_coffee_serialization() {
        let coffee = PopularCoffee {
            coffee_id: 1,
            coffee_name: "Espresso".to_string(),
            order_count: 100,
            average_rating: Some(Decimal::from_str("4.5").unwrap()),
            trend_percentage: Some(Decimal::from_str("15.5").unwrap()),
        };
        
        let json = serde_json::to_value(&coffee).unwrap();
        assert!(json.get("coffeeId").is_some());
        assert!(json.get("coffeeName").is_some());
        assert!(json.get("orderCount").is_some());
        assert!(json.get("averageRating").is_some());
        assert!(json.get("trendPercentage").is_some());
    }

    #[test]
    fn test_revenue_by_period_serialization() {
        let revenue = RevenueByPeriod {
            period: "2024-01".to_string(),
            revenue: Decimal::from_str("1234.56").unwrap(),
            timestamp: Utc::now(),
        };
        
        let json = serde_json::to_value(&revenue).unwrap();
        assert!(json.get("revenue").is_some());
        // Verify decimal precision is maintained
        let revenue_str = json["revenue"].to_string();
        assert!(revenue_str.contains("1234.56"));
    }

    #[test]
    fn test_rating_statistics_serialization() {
        let stats = RatingStatistics {
            average_rating: Decimal::from_str("4.25").unwrap(),
            total_reviews: 200,
            coffee_id: Some(5),
        };
        
        let json = serde_json::to_value(&stats).unwrap();
        assert!(json.get("averageRating").is_some());
        assert!(json.get("totalReviews").is_some());
        assert!(json.get("coffeeId").is_some());
    }

    #[test]
    fn test_response_metadata_serialization() {
        let metadata = ResponseMetadata::new(serde_json::json!({"limit": 10}))
            .with_result_count(50)
            .with_execution_time(125);
        
        let json = serde_json::to_value(&metadata).unwrap();
        assert!(json.get("queryParams").is_some());
        assert!(json.get("resultCount").is_some());
        assert!(json.get("executionTimeMs").is_some());
        assert_eq!(json["resultCount"], 50);
        assert_eq!(json["executionTimeMs"], 125);
    }

    // ============================================================================
    // TimePeriod Tests
    // ============================================================================

    #[test]
    fn test_time_period_serialization() {
        assert_eq!(
            serde_json::to_string(&TimePeriod::Daily).unwrap(),
            "\"daily\""
        );
        assert_eq!(
            serde_json::to_string(&TimePeriod::Weekly).unwrap(),
            "\"weekly\""
        );
        assert_eq!(
            serde_json::to_string(&TimePeriod::Monthly).unwrap(),
            "\"monthly\""
        );
        assert_eq!(
            serde_json::to_string(&TimePeriod::Custom).unwrap(),
            "\"custom\""
        );
    }

    #[test]
    fn test_time_period_deserialization() {
        let daily: TimePeriod = serde_json::from_str("\"daily\"").unwrap();
        assert_eq!(daily, TimePeriod::Daily);
        
        let weekly: TimePeriod = serde_json::from_str("\"weekly\"").unwrap();
        assert_eq!(weekly, TimePeriod::Weekly);
        
        let monthly: TimePeriod = serde_json::from_str("\"monthly\"").unwrap();
        assert_eq!(monthly, TimePeriod::Monthly);
        
        let custom: TimePeriod = serde_json::from_str("\"custom\"").unwrap();
        assert_eq!(custom, TimePeriod::Custom);
    }
}
