use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc};
use crate::analytics::types::{DateRange, TimePeriod};

/// Utility for handling time period filtering in analytics queries
pub struct TimePeriodFilter;

impl TimePeriodFilter {
    /// Parse and validate a date range, applying defaults if not specified
    /// Returns a validated DateRange with UTC timestamps
    pub fn parse_date_range(
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<DateRange, String> {
        let end = end_date.unwrap_or_else(Utc::now);
        let start = start_date.unwrap_or_else(|| end - Duration::days(30));

        let range = DateRange {
            start_date: start,
            end_date: end,
        };

        range.validate()?;
        Ok(range)
    }

    /// Calculate period boundaries for a given granularity
    /// Returns a vector of (period_start, period_end) tuples
    pub fn calculate_period_boundaries(
        date_range: &DateRange,
        period: TimePeriod,
    ) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
        let mut boundaries = Vec::new();
        let mut current = date_range.start_date;

        while current < date_range.end_date {
            let next = match period {
                TimePeriod::Daily => Self::next_day(current),
                TimePeriod::Weekly => Self::next_week(current),
                TimePeriod::Monthly => Self::next_month(current),
                TimePeriod::Custom => date_range.end_date,
            };

            let period_end = if next > date_range.end_date {
                date_range.end_date
            } else {
                next
            };

            boundaries.push((current, period_end));

            if period == TimePeriod::Custom {
                break;
            }

            current = next;
        }

        boundaries
    }

    /// Get the start of the day in UTC
    pub fn start_of_day(dt: DateTime<Utc>) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0)
            .single()
            .unwrap_or(dt)
    }

    /// Get the start of the week (Monday) in UTC
    pub fn start_of_week(dt: DateTime<Utc>) -> DateTime<Utc> {
        let days_from_monday = dt.weekday().num_days_from_monday();
        let start = dt - Duration::days(days_from_monday as i64);
        Self::start_of_day(start)
    }

    /// Get the start of the month in UTC
    pub fn start_of_month(dt: DateTime<Utc>) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(dt.year(), dt.month(), 1, 0, 0, 0)
            .single()
            .unwrap_or(dt)
    }

    /// Get the next day
    fn next_day(dt: DateTime<Utc>) -> DateTime<Utc> {
        dt + Duration::days(1)
    }

    /// Get the next week
    fn next_week(dt: DateTime<Utc>) -> DateTime<Utc> {
        dt + Duration::weeks(1)
    }

    /// Get the next month
    fn next_month(dt: DateTime<Utc>) -> DateTime<Utc> {
        let year = dt.year();
        let month = dt.month();
        
        if month == 12 {
            Utc.with_ymd_and_hms(year + 1, 1, dt.day(), dt.hour(), dt.minute(), dt.second())
                .single()
                .unwrap_or_else(|| dt + Duration::days(31))
        } else {
            Utc.with_ymd_and_hms(year, month + 1, dt.day(), dt.hour(), dt.minute(), dt.second())
                .single()
                .unwrap_or_else(|| dt + Duration::days(30))
        }
    }

    /// Format a period label for display
    pub fn format_period_label(start: DateTime<Utc>, period: TimePeriod) -> String {
        match period {
            TimePeriod::Daily => start.format("%Y-%m-%d").to_string(),
            TimePeriod::Weekly => format!("Week of {}", start.format("%Y-%m-%d")),
            TimePeriod::Monthly => start.format("%Y-%m").to_string(),
            TimePeriod::Custom => format!("{} to {}", 
                start.format("%Y-%m-%d"), 
                (start + Duration::days(1)).format("%Y-%m-%d")
            ),
        }
    }

    /// Validate that a period parameter is valid
    pub fn validate_period(period: &str) -> Result<TimePeriod, String> {
        match period.to_lowercase().as_str() {
            "daily" => Ok(TimePeriod::Daily),
            "weekly" => Ok(TimePeriod::Weekly),
            "monthly" => Ok(TimePeriod::Monthly),
            "custom" => Ok(TimePeriod::Custom),
            _ => Err(format!("Invalid period parameter: {}. Must be one of: daily, weekly, monthly, custom", period)),
        }
    }

    /// Ensure all timestamps are in UTC
    pub fn ensure_utc(dt: DateTime<Utc>) -> DateTime<Utc> {
        // Already in UTC, but this function exists for consistency
        dt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_parse_date_range_with_defaults() {
        let result = TimePeriodFilter::parse_date_range(None, None);
        assert!(result.is_ok());
        
        let range = result.unwrap();
        let duration = range.end_date - range.start_date;
        assert_eq!(duration.num_days(), 30);
    }

    #[test]
    fn test_parse_date_range_validation() {
        let end = Utc::now();
        let start = end + Duration::days(1);
        
        let result = TimePeriodFilter::parse_date_range(Some(start), Some(end));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("start_date must be before end_date"));
    }

    #[test]
    fn test_parse_date_range_valid() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 31, 23, 59, 59).unwrap();
        
        let result = TimePeriodFilter::parse_date_range(Some(start), Some(end));
        assert!(result.is_ok());
        
        let range = result.unwrap();
        assert_eq!(range.start_date, start);
        assert_eq!(range.end_date, end);
    }

    #[test]
    fn test_calculate_period_boundaries_daily() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 4, 0, 0, 0).unwrap();
        let range = DateRange { start_date: start, end_date: end };
        
        let boundaries = TimePeriodFilter::calculate_period_boundaries(&range, TimePeriod::Daily);
        assert_eq!(boundaries.len(), 3);
        
        assert_eq!(boundaries[0].0, start);
        assert_eq!(boundaries[0].1, start + Duration::days(1));
    }

    #[test]
    fn test_calculate_period_boundaries_weekly() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 22, 0, 0, 0).unwrap();
        let range = DateRange { start_date: start, end_date: end };
        
        let boundaries = TimePeriodFilter::calculate_period_boundaries(&range, TimePeriod::Weekly);
        assert_eq!(boundaries.len(), 3);
        
        assert_eq!(boundaries[0].0, start);
        assert_eq!(boundaries[0].1, start + Duration::weeks(1));
    }

    #[test]
    fn test_calculate_period_boundaries_monthly() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 4, 1, 0, 0, 0).unwrap();
        let range = DateRange { start_date: start, end_date: end };
        
        let boundaries = TimePeriodFilter::calculate_period_boundaries(&range, TimePeriod::Monthly);
        assert_eq!(boundaries.len(), 3);
    }

    #[test]
    fn test_calculate_period_boundaries_custom() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 31, 0, 0, 0).unwrap();
        let range = DateRange { start_date: start, end_date: end };
        
        let boundaries = TimePeriodFilter::calculate_period_boundaries(&range, TimePeriod::Custom);
        assert_eq!(boundaries.len(), 1);
        assert_eq!(boundaries[0].0, start);
        assert_eq!(boundaries[0].1, end);
    }

    #[test]
    fn test_start_of_day() {
        let dt = Utc.with_ymd_and_hms(2024, 1, 15, 14, 30, 45).unwrap();
        let start = TimePeriodFilter::start_of_day(dt);
        
        assert_eq!(start.hour(), 0);
        assert_eq!(start.minute(), 0);
        assert_eq!(start.second(), 0);
        assert_eq!(start.day(), 15);
    }

    #[test]
    fn test_start_of_week() {
        // January 15, 2024 is a Monday
        let monday = Utc.with_ymd_and_hms(2024, 1, 15, 14, 30, 45).unwrap();
        let start = TimePeriodFilter::start_of_week(monday);
        
        assert_eq!(start.day(), 15);
        assert_eq!(start.hour(), 0);
        
        // January 17, 2024 is a Wednesday
        let wednesday = Utc.with_ymd_and_hms(2024, 1, 17, 14, 30, 45).unwrap();
        let start = TimePeriodFilter::start_of_week(wednesday);
        
        assert_eq!(start.day(), 15); // Should be Monday
    }

    #[test]
    fn test_start_of_month() {
        let dt = Utc.with_ymd_and_hms(2024, 1, 15, 14, 30, 45).unwrap();
        let start = TimePeriodFilter::start_of_month(dt);
        
        assert_eq!(start.day(), 1);
        assert_eq!(start.hour(), 0);
        assert_eq!(start.minute(), 0);
        assert_eq!(start.second(), 0);
    }

    #[test]
    fn test_format_period_label_daily() {
        let dt = Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap();
        let label = TimePeriodFilter::format_period_label(dt, TimePeriod::Daily);
        assert_eq!(label, "2024-01-15");
    }

    #[test]
    fn test_format_period_label_weekly() {
        let dt = Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap();
        let label = TimePeriodFilter::format_period_label(dt, TimePeriod::Weekly);
        assert_eq!(label, "Week of 2024-01-15");
    }

    #[test]
    fn test_format_period_label_monthly() {
        let dt = Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap();
        let label = TimePeriodFilter::format_period_label(dt, TimePeriod::Monthly);
        assert_eq!(label, "2024-01");
    }

    #[test]
    fn test_validate_period_valid() {
        assert!(matches!(TimePeriodFilter::validate_period("daily"), Ok(TimePeriod::Daily)));
        assert!(matches!(TimePeriodFilter::validate_period("weekly"), Ok(TimePeriod::Weekly)));
        assert!(matches!(TimePeriodFilter::validate_period("monthly"), Ok(TimePeriod::Monthly)));
        assert!(matches!(TimePeriodFilter::validate_period("custom"), Ok(TimePeriod::Custom)));
        
        // Case insensitive
        assert!(matches!(TimePeriodFilter::validate_period("DAILY"), Ok(TimePeriod::Daily)));
    }

    #[test]
    fn test_validate_period_invalid() {
        let result = TimePeriodFilter::validate_period("invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid period parameter"));
    }

    #[test]
    fn test_leap_year_handling() {
        // February 29, 2024 (leap year)
        let leap_day = Utc.with_ymd_and_hms(2024, 2, 29, 0, 0, 0).unwrap();
        let next = TimePeriodFilter::next_month(leap_day);
        
        // Should handle leap year correctly
        assert_eq!(next.month(), 3);
    }

    #[test]
    fn test_year_boundary() {
        let dec = Utc.with_ymd_and_hms(2024, 12, 15, 0, 0, 0).unwrap();
        let next = TimePeriodFilter::next_month(dec);
        
        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 1);
    }

    #[test]
    fn test_ensure_utc() {
        let dt = Utc::now();
        let result = TimePeriodFilter::ensure_utc(dt);
        assert_eq!(dt, result);
    }
}
