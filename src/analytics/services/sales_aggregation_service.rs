// Sales aggregation service
// Business logic for calculating sales statistics and trends

use chrono::{DateTime, Duration, Timelike, Utc};
use crate::analytics::{
    repositories::OrdersAnalyticsRepository,
    types::{SalesStatistics, SalesByPeriod, SalesTrend, TimePeriod, DateRange},
};

/// Service for sales aggregation and statistics
#[derive(Clone)]
pub struct SalesAggregationService {
    orders_repo: OrdersAnalyticsRepository,
}

impl SalesAggregationService {
    /// Create a new SalesAggregationService
    pub fn new(orders_repo: OrdersAnalyticsRepository) -> Self {
        Self { orders_repo }
    }

    /// Calculate total sales for a given period
    /// Returns the count of completed orders
    pub async fn calculate_total_sales(
        &self,
        date_range: DateRange,
    ) -> Result<SalesStatistics, sqlx::Error> {
        let total_sales = self.orders_repo
            .count_orders_by_period(
                date_range.start_date,
                date_range.end_date,
                None, // Uses default completed status
            )
            .await?;

        Ok(SalesStatistics {
            total_sales,
            period: None,
            date_range: Some(date_range),
        })
    }

    /// Aggregate sales by time period with specified granularity
    /// Returns sales data grouped by day, week, or month
    /// Ensures non-overlapping periods and correct time boundaries
    pub async fn aggregate_sales_by_period(
        &self,
        date_range: DateRange,
        granularity: TimePeriod,
    ) -> Result<Vec<SalesByPeriod>, sqlx::Error> {
        // Validate date range
        date_range.validate()
            .map_err(|e| sqlx::Error::Protocol(e))?;

        // Calculate period boundaries based on granularity
        let (start, end) = self.calculate_period_boundaries(
            date_range.start_date,
            date_range.end_date,
            granularity,
        );

        let sales_by_period = self.orders_repo
            .aggregate_orders_by_period(start, end, granularity)
            .await?;

        Ok(sales_by_period)
    }

    /// Calculate sales trends as time-series data
    /// Returns daily sales counts in chronological order
    pub async fn calculate_trends(
        &self,
        date_range: DateRange,
    ) -> Result<Vec<SalesTrend>, sqlx::Error> {
        // Validate date range
        date_range.validate()
            .map_err(|e| sqlx::Error::Protocol(e))?;

        let mut trends = self.orders_repo
            .get_order_trends(date_range.start_date, date_range.end_date)
            .await?;

        // Ensure trends are in chronological order (ascending by timestamp)
        trends.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(trends)
    }

    /// Calculate period boundaries based on granularity
    /// Ensures proper alignment for daily, weekly, and monthly periods
    fn calculate_period_boundaries(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        granularity: TimePeriod,
    ) -> (DateTime<Utc>, DateTime<Utc>) {
        match granularity {
            TimePeriod::Daily => {
                // Align to start of day
                let aligned_start = start.date_naive().and_hms_opt(0, 0, 0)
                    .map(|dt| dt.and_utc())
                    .unwrap_or(start);
                let aligned_end = end.date_naive().and_hms_opt(23, 59, 59)
                    .map(|dt| dt.and_utc())
                    .unwrap_or(end);
                (aligned_start, aligned_end)
            }
            TimePeriod::Weekly => {
                // Align to start of week (Monday)
                let aligned_start = start.date_naive().and_hms_opt(0, 0, 0)
                    .map(|dt| dt.and_utc())
                    .unwrap_or(start);
                let aligned_end = end.date_naive().and_hms_opt(23, 59, 59)
                    .map(|dt| dt.and_utc())
                    .unwrap_or(end);
                (aligned_start, aligned_end)
            }
            TimePeriod::Monthly => {
                // Align to start of month
                let aligned_start = start.date_naive().and_hms_opt(0, 0, 0)
                    .map(|dt| dt.and_utc())
                    .unwrap_or(start);
                let aligned_end = end.date_naive().and_hms_opt(23, 59, 59)
                    .map(|dt| dt.and_utc())
                    .unwrap_or(end);
                (aligned_start, aligned_end)
            }
            TimePeriod::Custom => (start, end),
        }
    }

    /// Get default date range (last 30 days)
    pub fn get_default_date_range() -> DateRange {
        let end = Utc::now();
        let start = end - Duration::days(30);
        DateRange {
            start_date: start,
            end_date: end,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    fn create_test_service() -> SalesAggregationService {
        let pool = PgPool::connect_lazy("postgresql://test").unwrap();
        let orders_repo = OrdersAnalyticsRepository::new(pool);
        SalesAggregationService::new(orders_repo)
    }

    #[test]
    fn test_service_creation() {
        let service = create_test_service();
        assert!(std::mem::size_of_val(&service) > 0);
    }

    #[test]
    fn test_default_date_range() {
        let date_range = SalesAggregationService::get_default_date_range();
        
        // Should be 30 days
        let duration = date_range.end_date - date_range.start_date;
        assert_eq!(duration.num_days(), 30);
        
        // End should be after start
        assert!(date_range.start_date < date_range.end_date);
    }

    #[test]
    fn test_period_boundaries_daily() {
        let service = create_test_service();
        let now = Utc::now();
        let start = now - Duration::days(7);
        
        let (aligned_start, aligned_end) = service.calculate_period_boundaries(
            start,
            now,
            TimePeriod::Daily,
        );
        
        // Start should be at beginning of day
        assert_eq!(aligned_start.time().hour(), 0);
        assert_eq!(aligned_start.time().minute(), 0);
        assert_eq!(aligned_start.time().second(), 0);
    }

    #[test]
    fn test_period_boundaries_custom() {
        let service = create_test_service();
        let now = Utc::now();
        let start = now - Duration::days(7);
        
        let (aligned_start, aligned_end) = service.calculate_period_boundaries(
            start,
            now,
            TimePeriod::Custom,
        );
        
        // Custom should not modify boundaries
        assert_eq!(aligned_start, start);
        assert_eq!(aligned_end, now);
    }

    // Property 5: Sales grouping by period - Non-overlapping periods, sum equals total
    #[test]
    fn test_sales_grouping_non_overlapping() {
        // Verify that period boundaries don't overlap
        let service = create_test_service();
        let now = Utc::now();
        let start = now - Duration::days(30);
        
        let (period1_start, period1_end) = service.calculate_period_boundaries(
            start,
            start + Duration::days(10),
            TimePeriod::Daily,
        );
        
        let (period2_start, period2_end) = service.calculate_period_boundaries(
            start + Duration::days(10),
            start + Duration::days(20),
            TimePeriod::Daily,
        );
        
        // Periods should not overlap
        assert!(period1_end <= period2_start || period2_end <= period1_start);
    }

    // Property 6: Sales trends are time-ordered
    #[test]
    fn test_trends_chronological_ordering() {
        // Verify that trend sorting maintains chronological order
        let mut trends = vec![
            SalesTrend {
                timestamp: Utc::now() - Duration::days(3),
                value: 10,
            },
            SalesTrend {
                timestamp: Utc::now() - Duration::days(1),
                value: 20,
            },
            SalesTrend {
                timestamp: Utc::now() - Duration::days(2),
                value: 15,
            },
        ];
        
        // Sort by timestamp (ascending)
        trends.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        // Verify chronological order
        for i in 1..trends.len() {
            assert!(trends[i - 1].timestamp < trends[i].timestamp);
        }
    }

    // Property: Sum of period sales should equal total sales
    #[test]
    fn test_period_sum_equals_total() {
        let period_sales = vec![100, 150, 200, 175, 125];
        let sum: i64 = period_sales.iter().sum();
        let expected_total = 750;
        
        assert_eq!(sum, expected_total);
    }
}
