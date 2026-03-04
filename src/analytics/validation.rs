// Analytics input validation utilities
// Provides validation for query parameters, time periods, limits, and IDs

use chrono::{DateTime, Utc};
use crate::analytics::{error::AnalyticsError, types::TimePeriod};

/// Validator for analytics query parameters
pub struct AnalyticsValidator;

impl AnalyticsValidator {
    /// Validate date range parameters
    /// Ensures start_date is before end_date
    pub fn validate_date_range(
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<(), AnalyticsError> {
        if let (Some(start), Some(end)) = (start_date, end_date) {
            if start >= end {
                return Err(AnalyticsError::validation(
                    "dateRange",
                    &format!("start: {}, end: {}", start, end),
                    "start_date must be before end_date",
                ));
            }
        }
        Ok(())
    }

    /// Validate time period parameter
    /// Ensures period is one of: daily, weekly, monthly, custom
    pub fn validate_period(period: &str) -> Result<TimePeriod, AnalyticsError> {
        match period.to_lowercase().as_str() {
            "daily" => Ok(TimePeriod::Daily),
            "weekly" => Ok(TimePeriod::Weekly),
            "monthly" => Ok(TimePeriod::Monthly),
            "custom" => Ok(TimePeriod::Custom),
            _ => Err(AnalyticsError::validation(
                "period",
                period,
                "one of: daily, weekly, monthly, custom",
            )),
        }
    }

    /// Validate limit parameter
    /// Ensures limit is positive and within reasonable bounds (1-1000)
    pub fn validate_limit(limit: Option<i32>) -> Result<i32, AnalyticsError> {
        match limit {
            None => Ok(10), // Default limit
            Some(l) if l < 1 => Err(AnalyticsError::validation(
                "limit",
                &l.to_string(),
                "positive integer (minimum: 1)",
            )),
            Some(l) if l > 1000 => Err(AnalyticsError::validation(
                "limit",
                &l.to_string(),
                "integer between 1 and 1000 (maximum: 1000)",
            )),
            Some(l) => Ok(l),
        }
    }

    /// Validate coffee ID parameter
    /// Ensures coffee_id is positive
    pub fn validate_coffee_id(coffee_id: Option<i32>) -> Result<Option<i32>, AnalyticsError> {
        match coffee_id {
            None => Ok(None),
            Some(id) if id < 1 => Err(AnalyticsError::validation(
                "coffeeId",
                &id.to_string(),
                "positive integer (minimum: 1)",
            )),
            Some(id) => Ok(Some(id)),
        }
    }

    /// Validate that a date is not in the future
    pub fn validate_not_future(date: DateTime<Utc>, field_name: &str) -> Result<(), AnalyticsError> {
        let now = Utc::now();
        if date > now {
            return Err(AnalyticsError::validation(
                field_name,
                &date.to_rfc3339(),
                "date must not be in the future",
            ));
        }
        Ok(())
    }

    /// Validate date range span (not too large)
    /// Maximum span: 1 year (365 days)
    pub fn validate_date_range_span(
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<(), AnalyticsError> {
        let duration = end_date.signed_duration_since(start_date);
        let max_days = 365;
        
        if duration.num_days() > max_days {
            return Err(AnalyticsError::validation(
                "dateRange",
                &format!("{} days", duration.num_days()),
                &format!("maximum span of {} days", max_days),
            ));
        }
        
        Ok(())
    }

    /// Validate all query parameters at once
    pub fn validate_query_params(
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        period: Option<&str>,
        limit: Option<i32>,
        coffee_id: Option<i32>,
    ) -> Result<(), AnalyticsError> {
        // Validate date range
        Self::validate_date_range(start_date, end_date)?;
        
        // Validate dates are not in the future
        if let Some(start) = start_date {
            Self::validate_not_future(start, "startDate")?;
        }
        if let Some(end) = end_date {
            Self::validate_not_future(end, "endDate")?;
        }
        
        // Validate date range span
        if let (Some(start), Some(end)) = (start_date, end_date) {
            Self::validate_date_range_span(start, end)?;
        }
        
        // Validate period
        if let Some(p) = period {
            Self::validate_period(p)?;
        }
        
        // Validate limit
        Self::validate_limit(limit)?;
        
        // Validate coffee_id
        Self::validate_coffee_id(coffee_id)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_validate_date_range_valid() {
        let start = Utc::now() - Duration::days(7);
        let end = Utc::now();
        
        let result = AnalyticsValidator::validate_date_range(Some(start), Some(end));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_date_range_invalid() {
        let start = Utc::now();
        let end = Utc::now() - Duration::days(7);
        
        let result = AnalyticsValidator::validate_date_range(Some(start), Some(end));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_date_range_none() {
        let result = AnalyticsValidator::validate_date_range(None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_period_valid() {
        let periods = vec!["daily", "weekly", "monthly", "custom", "DAILY", "Weekly"];
        
        for period in periods {
            let result = AnalyticsValidator::validate_period(period);
            assert!(result.is_ok(), "Failed for period: {}", period);
        }
    }

    #[test]
    fn test_validate_period_invalid() {
        let result = AnalyticsValidator::validate_period("yearly");
        assert!(result.is_err());
        
        match result {
            Err(AnalyticsError::ValidationError { field, value, expected, .. }) => {
                assert_eq!(field, "period");
                assert_eq!(value, "yearly");
                assert!(expected.contains("daily"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_validate_limit_valid() {
        let valid_limits = vec![Some(1), Some(10), Some(100), Some(1000)];
        
        for limit in valid_limits {
            let result = AnalyticsValidator::validate_limit(limit);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_validate_limit_default() {
        let result = AnalyticsValidator::validate_limit(None);
        assert_eq!(result.unwrap(), 10);
    }

    #[test]
    fn test_validate_limit_too_small() {
        let result = AnalyticsValidator::validate_limit(Some(0));
        assert!(result.is_err());
        
        let result = AnalyticsValidator::validate_limit(Some(-5));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_limit_too_large() {
        let result = AnalyticsValidator::validate_limit(Some(1001));
        assert!(result.is_err());
        
        match result {
            Err(AnalyticsError::ValidationError { field, value, expected, .. }) => {
                assert_eq!(field, "limit");
                assert_eq!(value, "1001");
                assert!(expected.contains("1000"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_validate_coffee_id_valid() {
        let result = AnalyticsValidator::validate_coffee_id(Some(1));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(1));
        
        let result = AnalyticsValidator::validate_coffee_id(None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_validate_coffee_id_invalid() {
        let result = AnalyticsValidator::validate_coffee_id(Some(0));
        assert!(result.is_err());
        
        let result = AnalyticsValidator::validate_coffee_id(Some(-1));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_not_future() {
        let past = Utc::now() - Duration::days(1);
        let result = AnalyticsValidator::validate_not_future(past, "testDate");
        assert!(result.is_ok());
        
        let future = Utc::now() + Duration::days(1);
        let result = AnalyticsValidator::validate_not_future(future, "testDate");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_date_range_span_valid() {
        let start = Utc::now() - Duration::days(30);
        let end = Utc::now();
        
        let result = AnalyticsValidator::validate_date_range_span(start, end);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_date_range_span_too_large() {
        let start = Utc::now() - Duration::days(400);
        let end = Utc::now();
        
        let result = AnalyticsValidator::validate_date_range_span(start, end);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_query_params_all_valid() {
        let start = Utc::now() - Duration::days(7);
        let end = Utc::now();
        
        let result = AnalyticsValidator::validate_query_params(
            Some(start),
            Some(end),
            Some("daily"),
            Some(10),
            Some(5),
        );
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_query_params_invalid_period() {
        let start = Utc::now() - Duration::days(7);
        let end = Utc::now();
        
        let result = AnalyticsValidator::validate_query_params(
            Some(start),
            Some(end),
            Some("invalid"),
            Some(10),
            None,
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_query_params_invalid_limit() {
        let start = Utc::now() - Duration::days(7);
        let end = Utc::now();
        
        let result = AnalyticsValidator::validate_query_params(
            Some(start),
            Some(end),
            Some("daily"),
            Some(-5),
            None,
        );
        
        assert!(result.is_err());
    }

    // Property 28: Validation error details - Include field, value, expected format
    #[test]
    fn test_validation_error_includes_all_details() {
        let result = AnalyticsValidator::validate_limit(Some(2000));
        
        match result {
            Err(AnalyticsError::ValidationError { field, value, expected, message }) => {
                // All fields should be present and non-empty
                assert!(!field.is_empty());
                assert!(!value.is_empty());
                assert!(!expected.is_empty());
                assert!(!message.is_empty());
                
                // Field should be "limit"
                assert_eq!(field, "limit");
                
                // Value should be the actual invalid value
                assert_eq!(value, "2000");
                
                // Expected should describe the valid format
                assert!(expected.contains("1000"));
            }
            _ => panic!("Expected ValidationError with all details"),
        }
    }

    #[test]
    fn test_validation_error_message_clarity() {
        let result = AnalyticsValidator::validate_period("yearly");
        
        match result {
            Err(AnalyticsError::ValidationError { message, .. }) => {
                // Message should be clear and helpful
                assert!(message.contains("period"));
                assert!(message.contains("yearly"));
                assert!(message.contains("daily") || message.contains("weekly"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }
}
