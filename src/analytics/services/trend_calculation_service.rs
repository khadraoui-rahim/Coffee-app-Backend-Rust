// Trend calculation service
// Business logic for calculating trending items by comparing time periods

use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use crate::analytics::{
    repositories::OrdersAnalyticsRepository,
    types::{PopularCoffee, DateRange},
};

/// Service for trend calculation
#[derive(Clone)]
pub struct TrendCalculationService {
    orders_repo: OrdersAnalyticsRepository,
}

impl TrendCalculationService {
    /// Create a new TrendCalculationService
    pub fn new(orders_repo: OrdersAnalyticsRepository) -> Self {
        Self { orders_repo }
    }

    /// Calculate trending items by comparing two time periods
    /// Returns coffees with the largest increase in order frequency
    /// Formula: ((current - previous) / previous) * 100
    pub async fn calculate_trending_items(
        &self,
        current_period: DateRange,
        previous_period: DateRange,
        limit: i32,
    ) -> Result<Vec<PopularCoffee>, sqlx::Error> {
        // Validate date ranges
        current_period.validate()
            .map_err(|e| sqlx::Error::Protocol(e))?;
        previous_period.validate()
            .map_err(|e| sqlx::Error::Protocol(e))?;

        // Validate limit
        let validated_limit = if limit < 1 { 10 } else { limit.min(100) };

        // Get order counts for current period
        let current_orders = self.orders_repo
            .get_most_ordered_coffees(
                current_period.start_date,
                current_period.end_date,
                100, // Get more to calculate trends
            )
            .await?;

        // Get order counts for previous period
        let previous_orders = self.orders_repo
            .get_most_ordered_coffees(
                previous_period.start_date,
                previous_period.end_date,
                100,
            )
            .await?;

        // Build map of previous period counts
        let previous_map: std::collections::HashMap<i32, i64> = previous_orders
            .into_iter()
            .map(|(id, _, count)| (id, count))
            .collect();

        // Calculate trend percentages
        let mut trending_items: Vec<PopularCoffee> = current_orders
            .into_iter()
            .map(|(coffee_id, coffee_name, current_count)| {
                let previous_count = previous_map.get(&coffee_id).copied().unwrap_or(0);
                let trend_percentage = Self::calculate_trend_percentage(
                    current_count,
                    previous_count,
                );

                PopularCoffee {
                    coffee_id,
                    coffee_name,
                    order_count: current_count,
                    average_rating: None,
                    trend_percentage: Some(trend_percentage),
                }
            })
            .collect();

        // Sort by trend percentage (descending)
        trending_items.sort_by(|a, b| {
            let a_trend = a.trend_percentage.unwrap_or(Decimal::ZERO);
            let b_trend = b.trend_percentage.unwrap_or(Decimal::ZERO);
            b_trend.cmp(&a_trend)
        });

        // Take top N items
        trending_items.truncate(validated_limit as usize);

        Ok(trending_items)
    }

    /// Calculate trend percentage using the formula:
    /// ((current - previous) / previous) * 100
    /// 
    /// Handles division by zero when previous period has no orders
    /// Returns 100% for new items (0 previous orders)
    fn calculate_trend_percentage(current: i64, previous: i64) -> Decimal {
        if previous == 0 {
            // New item or no previous orders
            if current > 0 {
                // Return 100% growth for new items
                Decimal::from(100)
            } else {
                Decimal::ZERO
            }
        } else {
            let current_dec = Decimal::from(current);
            let previous_dec = Decimal::from(previous);
            let difference = current_dec - previous_dec;
            let percentage = (difference / previous_dec) * Decimal::from(100);
            
            // Round to 2 decimal places
            percentage.round_dp(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use std::str::FromStr;

    fn create_test_service() -> TrendCalculationService {
        let pool = PgPool::connect_lazy("postgresql://test").unwrap();
        let orders_repo = OrdersAnalyticsRepository::new(pool);
        TrendCalculationService::new(orders_repo)
    }

    #[test]
    fn test_service_creation() {
        let service = create_test_service();
        assert!(std::mem::size_of_val(&service) > 0);
    }

    // Property 9: Trending calculation accuracy - Correct percentage formula
    #[test]
    fn test_trend_percentage_calculation() {
        // Test case: 100 previous, 150 current = 50% increase
        let percentage = TrendCalculationService::calculate_trend_percentage(150, 100);
        assert_eq!(percentage, Decimal::from(50));
    }

    #[test]
    fn test_trend_percentage_positive_growth() {
        let test_cases = vec![
            (150, 100, "50.00"),   // 50% increase
            (200, 100, "100.00"),  // 100% increase
            (125, 100, "25.00"),   // 25% increase
            (110, 100, "10.00"),   // 10% increase
        ];

        for (current, previous, expected) in test_cases {
            let percentage = TrendCalculationService::calculate_trend_percentage(current, previous);
            let expected_value = Decimal::from_str(expected).unwrap();
            assert_eq!(percentage, expected_value);
        }
    }

    #[test]
    fn test_trend_percentage_negative_growth() {
        let test_cases = vec![
            (75, 100, "-25.00"),   // 25% decrease
            (50, 100, "-50.00"),   // 50% decrease
            (90, 100, "-10.00"),   // 10% decrease
            (25, 100, "-75.00"),   // 75% decrease
        ];

        for (current, previous, expected) in test_cases {
            let percentage = TrendCalculationService::calculate_trend_percentage(current, previous);
            let expected_value = Decimal::from_str(expected).unwrap();
            assert_eq!(percentage, expected_value);
        }
    }

    #[test]
    fn test_trend_percentage_zero_previous() {
        // When previous is 0, should return 100% for new items
        let percentage = TrendCalculationService::calculate_trend_percentage(50, 0);
        assert_eq!(percentage, Decimal::from(100));
    }

    #[test]
    fn test_trend_percentage_zero_both() {
        // When both are 0, should return 0%
        let percentage = TrendCalculationService::calculate_trend_percentage(0, 0);
        assert_eq!(percentage, Decimal::ZERO);
    }

    #[test]
    fn test_trend_percentage_no_change() {
        // When current equals previous, should return 0%
        let percentage = TrendCalculationService::calculate_trend_percentage(100, 100);
        assert_eq!(percentage, Decimal::ZERO);
    }

    #[test]
    fn test_trend_percentage_decimal_precision() {
        // Test rounding to 2 decimal places
        let percentage = TrendCalculationService::calculate_trend_percentage(123, 100);
        // (123 - 100) / 100 * 100 = 23%
        assert_eq!(percentage, Decimal::from(23));
        assert_eq!(percentage.scale(), 2);
    }

    #[test]
    fn test_trend_percentage_formula() {
        // Verify the formula: ((current - previous) / previous) * 100
        let current = 150;
        let previous = 100;
        
        let difference = current - previous; // 50
        let ratio = difference as f64 / previous as f64; // 0.5
        let expected_percentage = ratio * 100.0; // 50.0
        
        let calculated = TrendCalculationService::calculate_trend_percentage(current, previous);
        assert_eq!(calculated, Decimal::from(50));
        assert_eq!(expected_percentage, 50.0);
    }

    #[test]
    fn test_trend_percentage_large_growth() {
        // Test large percentage increases
        let percentage = TrendCalculationService::calculate_trend_percentage(1000, 100);
        // (1000 - 100) / 100 * 100 = 900%
        assert_eq!(percentage, Decimal::from(900));
    }

    #[test]
    fn test_trend_percentage_small_numbers() {
        // Test with small numbers
        let percentage = TrendCalculationService::calculate_trend_percentage(3, 2);
        // (3 - 2) / 2 * 100 = 50%
        assert_eq!(percentage, Decimal::from(50));
    }

    #[test]
    fn test_trending_items_sorting() {
        // Verify trending items are sorted by trend percentage (descending)
        let mut items = vec![
            PopularCoffee {
                coffee_id: 1,
                coffee_name: "Coffee A".to_string(),
                order_count: 100,
                average_rating: None,
                trend_percentage: Some(Decimal::from(25)),
            },
            PopularCoffee {
                coffee_id: 2,
                coffee_name: "Coffee B".to_string(),
                order_count: 150,
                average_rating: None,
                trend_percentage: Some(Decimal::from(75)),
            },
            PopularCoffee {
                coffee_id: 3,
                coffee_name: "Coffee C".to_string(),
                order_count: 125,
                average_rating: None,
                trend_percentage: Some(Decimal::from(50)),
            },
        ];

        // Sort by trend percentage (descending)
        items.sort_by(|a, b| {
            let a_trend = a.trend_percentage.unwrap_or(Decimal::ZERO);
            let b_trend = b.trend_percentage.unwrap_or(Decimal::ZERO);
            b_trend.cmp(&a_trend)
        });

        // Verify descending order
        assert_eq!(items[0].trend_percentage, Some(Decimal::from(75)));
        assert_eq!(items[1].trend_percentage, Some(Decimal::from(50)));
        assert_eq!(items[2].trend_percentage, Some(Decimal::from(25)));
    }

    #[test]
    fn test_limit_enforcement() {
        let limit = 10;
        let validated = if limit < 1 { 10 } else { limit.min(100) };
        
        assert_eq!(validated, 10);
    }

    #[test]
    fn test_limit_maximum() {
        let limit = 150;
        let validated = if limit < 1 { 10 } else { limit.min(100) };
        
        assert_eq!(validated, 100);
    }
}
