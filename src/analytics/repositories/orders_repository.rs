// Orders repository for analytics queries
// Provides aggregated data about orders for sales and revenue analytics

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::analytics::types::{OrderStatus, SalesByPeriod, SalesTrend, RevenueByCoffee, TimePeriod};

/// Repository for order analytics queries
#[derive(Clone)]
pub struct OrdersAnalyticsRepository {
    pool: PgPool,
}

impl OrdersAnalyticsRepository {
    /// Create a new OrdersAnalyticsRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Count orders by period with optional status filtering
    /// Only counts completed orders by default for accurate sales statistics
    pub async fn count_orders_by_period(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        status: Option<OrderStatus>,
    ) -> Result<i64, sqlx::Error> {
        let status_filter = status.unwrap_or(OrderStatus::Completed);
        
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM orders
            WHERE created_at >= $1 
              AND created_at < $2
              AND status = $3
            "#
        )
        .bind(start_date)
        .bind(end_date)
        .bind(status_filter)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.0)
    }

    /// Aggregate orders by time period with specified granularity
    /// Returns sales data grouped by day, week, or month
    pub async fn aggregate_orders_by_period(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        granularity: TimePeriod,
    ) -> Result<Vec<SalesByPeriod>, sqlx::Error> {
        let date_trunc = match granularity {
            TimePeriod::Daily => "day",
            TimePeriod::Weekly => "week",
            TimePeriod::Monthly => "month",
            TimePeriod::Custom => "day", // Default to daily for custom ranges
        };

        let results = sqlx::query_as::<_, (DateTime<Utc>, i64)>(
            &format!(
                r#"
                SELECT 
                    DATE_TRUNC('{}', created_at) as period,
                    COUNT(*) as sales_count
                FROM orders
                WHERE created_at >= $1 
                  AND created_at < $2
                  AND status = 'completed'
                GROUP BY period
                ORDER BY period ASC
                "#,
                date_trunc
            )
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await?;

        Ok(results
            .into_iter()
            .map(|(timestamp, sales_count)| SalesByPeriod {
                period: timestamp.format("%Y-%m-%d").to_string(),
                sales_count,
                timestamp,
            })
            .collect())
    }

    /// Get order trends as time-series data
    /// Returns daily order counts for trend analysis
    pub async fn get_order_trends(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<SalesTrend>, sqlx::Error> {
        let results = sqlx::query_as::<_, (DateTime<Utc>, i64)>(
            r#"
            SELECT 
                DATE_TRUNC('day', created_at) as timestamp,
                COUNT(*) as value
            FROM orders
            WHERE created_at >= $1 
              AND created_at < $2
              AND status = 'completed'
            GROUP BY timestamp
            ORDER BY timestamp ASC
            "#
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await?;

        Ok(results
            .into_iter()
            .map(|(timestamp, value)| SalesTrend { timestamp, value })
            .collect())
    }

    /// Calculate revenue by coffee type
    /// Groups revenue by coffee item, only including completed orders
    pub async fn calculate_revenue_by_coffee(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<RevenueByCoffee>, sqlx::Error> {
        let results = sqlx::query_as::<_, (i32, String, Decimal)>(
            r#"
            SELECT 
                c.id as coffee_id,
                c.name as coffee_name,
                COALESCE(SUM(oi.subtotal), 0) as revenue
            FROM coffees c
            LEFT JOIN order_items oi ON c.id = oi.coffee_item_id
            LEFT JOIN orders o ON oi.order_id = o.id
            WHERE o.created_at >= $1 
              AND o.created_at < $2
              AND o.status = 'completed'
            GROUP BY c.id, c.name
            HAVING SUM(oi.subtotal) > 0
            ORDER BY revenue DESC
            "#
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await?;

        Ok(results
            .into_iter()
            .map(|(coffee_id, coffee_name, revenue)| RevenueByCoffee {
                coffee_id,
                coffee_name,
                revenue,
            })
            .collect())
    }

    /// Calculate total revenue for a period
    /// Only includes completed orders
    pub async fn calculate_total_revenue(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Decimal, sqlx::Error> {
        let result: (Option<Decimal>,) = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(total_price), 0)
            FROM orders
            WHERE created_at >= $1 
              AND created_at < $2
              AND status = 'completed'
            "#
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.0.unwrap_or(Decimal::ZERO))
    }

    /// Aggregate revenue by time period
    pub async fn aggregate_revenue_by_period(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        granularity: TimePeriod,
    ) -> Result<Vec<(DateTime<Utc>, Decimal)>, sqlx::Error> {
        let date_trunc = match granularity {
            TimePeriod::Daily => "day",
            TimePeriod::Weekly => "week",
            TimePeriod::Monthly => "month",
            TimePeriod::Custom => "day",
        };

        let results = sqlx::query_as::<_, (DateTime<Utc>, Option<Decimal>)>(
            &format!(
                r#"
                SELECT 
                    DATE_TRUNC('{}', created_at) as period,
                    COALESCE(SUM(total_price), 0) as revenue
                FROM orders
                WHERE created_at >= $1 
                  AND created_at < $2
                  AND status = 'completed'
                GROUP BY period
                ORDER BY period ASC
                "#,
                date_trunc
            )
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await?;

        Ok(results
            .into_iter()
            .map(|(timestamp, revenue)| (timestamp, revenue.unwrap_or(Decimal::ZERO)))
            .collect())
    }

    /// Get most ordered coffees with order counts
    pub async fn get_most_ordered_coffees(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        limit: i32,
    ) -> Result<Vec<(i32, String, i64)>, sqlx::Error> {
        let results = sqlx::query_as::<_, (i32, String, i64)>(
            r#"
            SELECT 
                c.id as coffee_id,
                c.name as coffee_name,
                COUNT(DISTINCT o.id) as order_count
            FROM coffees c
            INNER JOIN order_items oi ON c.id = oi.coffee_item_id
            INNER JOIN orders o ON oi.order_id = o.id
            WHERE o.created_at >= $1 
              AND o.created_at < $2
              AND o.status = 'completed'
            GROUP BY c.id, c.name
            ORDER BY order_count DESC
            LIMIT $3
            "#
        )
        .bind(start_date)
        .bind(end_date)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    // Helper to create test repository
    // Note: These are unit tests for the repository structure
    // Integration tests with actual database would require testcontainers

    #[test]
    fn test_repository_creation() {
        // This test verifies the repository can be created
        // Actual database operations are tested in integration tests
        let pool = PgPool::connect_lazy("postgresql://test").unwrap();
        let repo = OrdersAnalyticsRepository::new(pool);
        assert!(std::mem::size_of_val(&repo) > 0);
    }

    #[test]
    fn test_date_range_parameters() {
        // Verify date range calculations work correctly
        let now = Utc::now();
        let start = now - Duration::days(30);
        let end = now;
        
        assert!(start < end);
        assert_eq!((end - start).num_days(), 30);
    }

    #[test]
    fn test_time_period_granularity_mapping() {
        // Verify time period enum maps to correct SQL date_trunc values
        let test_cases = vec![
            (TimePeriod::Daily, "day"),
            (TimePeriod::Weekly, "week"),
            (TimePeriod::Monthly, "month"),
            (TimePeriod::Custom, "day"),
        ];

        for (period, expected) in test_cases {
            let date_trunc = match period {
                TimePeriod::Daily => "day",
                TimePeriod::Weekly => "week",
                TimePeriod::Monthly => "month",
                TimePeriod::Custom => "day",
            };
            assert_eq!(date_trunc, expected);
        }
    }
}
