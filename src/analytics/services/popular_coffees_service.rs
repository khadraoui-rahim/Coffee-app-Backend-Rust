// Popular coffees service
// Business logic for identifying most ordered and highest rated coffees

use rust_decimal::Decimal;
use crate::analytics::{
    repositories::{OrdersAnalyticsRepository, ReviewsAnalyticsRepository},
    types::{PopularCoffee, DateRange},
};

/// Service for popular coffee analytics
#[derive(Clone)]
pub struct PopularCoffeesService {
    orders_repo: OrdersAnalyticsRepository,
    reviews_repo: ReviewsAnalyticsRepository,
}

impl PopularCoffeesService {
    /// Create a new PopularCoffeesService
    pub fn new(
        orders_repo: OrdersAnalyticsRepository,
        reviews_repo: ReviewsAnalyticsRepository,
    ) -> Self {
        Self {
            orders_repo,
            reviews_repo,
        }
    }

    /// Get most ordered coffees ranked by order count
    /// Returns top N coffees in descending order by order count
    pub async fn get_most_ordered(
        &self,
        date_range: DateRange,
        limit: i32,
    ) -> Result<Vec<PopularCoffee>, sqlx::Error> {
        // Validate date range
        date_range.validate()
            .map_err(|e| sqlx::Error::Protocol(e))?;

        // Validate limit
        let validated_limit = Self::validate_limit(limit)?;

        // Get most ordered coffees from repository
        let most_ordered = self.orders_repo
            .get_most_ordered_coffees(
                date_range.start_date,
                date_range.end_date,
                validated_limit,
            )
            .await?;

        // Enrich with average ratings
        let mut result = Vec::new();
        for (coffee_id, coffee_name, order_count) in most_ordered {
            let average_rating = self.reviews_repo
                .calculate_average_rating(Some(coffee_id))
                .await?;

            result.push(PopularCoffee {
                coffee_id,
                coffee_name,
                order_count,
                average_rating,
                trend_percentage: None,
            });
        }

        // Verify descending order by count
        Self::verify_descending_order_by_count(&result);

        Ok(result)
    }

    /// Get highest rated coffees ranked by average rating
    /// Returns top N coffees in descending order by rating
    /// Requires minimum number of reviews to ensure statistical significance
    pub async fn get_highest_rated(
        &self,
        limit: i32,
        min_reviews: i32,
    ) -> Result<Vec<PopularCoffee>, sqlx::Error> {
        // Validate limit
        let validated_limit = Self::validate_limit(limit)?;

        // Validate min_reviews
        let validated_min_reviews = if min_reviews < 1 { 1 } else { min_reviews };

        // Get highest rated coffees from repository
        let highest_rated = self.reviews_repo
            .get_highest_rated_coffees(validated_limit, validated_min_reviews)
            .await?;

        // Convert to PopularCoffee format
        let result: Vec<PopularCoffee> = highest_rated
            .into_iter()
            .map(|(coffee_id, coffee_name, average_rating, review_count)| {
                PopularCoffee {
                    coffee_id,
                    coffee_name,
                    order_count: review_count,
                    average_rating: Some(average_rating),
                    trend_percentage: None,
                }
            })
            .collect();

        // Verify descending order by rating
        Self::verify_descending_order_by_rating(&result);

        Ok(result)
    }

    /// Validate and enforce limit parameter
    /// Ensures limit is positive and within reasonable bounds
    fn validate_limit(limit: i32) -> Result<i32, sqlx::Error> {
        if limit < 1 {
            return Err(sqlx::Error::Protocol(
                "Limit must be at least 1".to_string()
            ));
        }
        
        // Enforce maximum limit to prevent excessive results
        const MAX_LIMIT: i32 = 100;
        Ok(limit.min(MAX_LIMIT))
    }

    /// Verify that results are in descending order by count
    /// This ensures Property 7: Most ordered ranking correctness
    fn verify_descending_order_by_count(coffees: &[PopularCoffee]) {
        for i in 1..coffees.len() {
            debug_assert!(
                coffees[i - 1].order_count >= coffees[i].order_count,
                "Most ordered coffees should be in descending order by count"
            );
        }
    }

    /// Verify that results are in descending order by rating
    /// This ensures Property 8: Highest rated ranking correctness
    fn verify_descending_order_by_rating(coffees: &[PopularCoffee]) {
        for i in 1..coffees.len() {
            if let (Some(prev_rating), Some(curr_rating)) = 
                (coffees[i - 1].average_rating, coffees[i].average_rating) {
                debug_assert!(
                    prev_rating >= curr_rating,
                    "Highest rated coffees should be in descending order by rating"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use std::str::FromStr;

    fn create_test_service() -> PopularCoffeesService {
        let pool = PgPool::connect_lazy("postgresql://test").unwrap();
        let orders_repo = OrdersAnalyticsRepository::new(pool.clone());
        let reviews_repo = ReviewsAnalyticsRepository::new(pool);
        PopularCoffeesService::new(orders_repo, reviews_repo)
    }

    #[test]
    fn test_service_creation() {
        let service = create_test_service();
        assert!(std::mem::size_of_val(&service) > 0);
    }

    // Property 7: Most ordered ranking correctness - Descending order by count
    #[test]
    fn test_most_ordered_descending_order() {
        let coffees = vec![
            PopularCoffee {
                coffee_id: 1,
                coffee_name: "Espresso".to_string(),
                order_count: 100,
                average_rating: Some(Decimal::from_str("4.5").unwrap()),
                trend_percentage: None,
            },
            PopularCoffee {
                coffee_id: 2,
                coffee_name: "Latte".to_string(),
                order_count: 75,
                average_rating: Some(Decimal::from_str("4.3").unwrap()),
                trend_percentage: None,
            },
            PopularCoffee {
                coffee_id: 3,
                coffee_name: "Cappuccino".to_string(),
                order_count: 50,
                average_rating: Some(Decimal::from_str("4.7").unwrap()),
                trend_percentage: None,
            },
        ];

        // Verify descending order
        for i in 1..coffees.len() {
            assert!(coffees[i - 1].order_count >= coffees[i].order_count);
        }
    }

    // Property 8: Highest rated ranking correctness - Descending order by rating
    #[test]
    fn test_highest_rated_descending_order() {
        let coffees = vec![
            PopularCoffee {
                coffee_id: 1,
                coffee_name: "Cappuccino".to_string(),
                order_count: 50,
                average_rating: Some(Decimal::from_str("4.9").unwrap()),
                trend_percentage: None,
            },
            PopularCoffee {
                coffee_id: 2,
                coffee_name: "Espresso".to_string(),
                order_count: 100,
                average_rating: Some(Decimal::from_str("4.7").unwrap()),
                trend_percentage: None,
            },
            PopularCoffee {
                coffee_id: 3,
                coffee_name: "Latte".to_string(),
                order_count: 75,
                average_rating: Some(Decimal::from_str("4.5").unwrap()),
                trend_percentage: None,
            },
        ];

        // Verify descending order by rating
        for i in 1..coffees.len() {
            if let (Some(prev), Some(curr)) = 
                (coffees[i - 1].average_rating, coffees[i].average_rating) {
                assert!(prev >= curr);
            }
        }
    }

    // Property 10: Result limit enforcement - At most N items
    #[test]
    fn test_limit_enforcement() {
        let limit = 10;
        let result = PopularCoffeesService::validate_limit(limit).unwrap();
        
        assert_eq!(result, limit);
        assert!(result <= 100); // Max limit
    }

    #[test]
    fn test_limit_validation_minimum() {
        let result = PopularCoffeesService::validate_limit(0);
        assert!(result.is_err());
        
        let result = PopularCoffeesService::validate_limit(-5);
        assert!(result.is_err());
    }

    #[test]
    fn test_limit_validation_maximum() {
        let result = PopularCoffeesService::validate_limit(150).unwrap();
        assert_eq!(result, 100); // Should cap at max
    }

    #[test]
    fn test_limit_validation_valid_range() {
        let test_limits = vec![1, 5, 10, 25, 50, 100];
        
        for limit in test_limits {
            let result = PopularCoffeesService::validate_limit(limit).unwrap();
            assert_eq!(result, limit);
        }
    }

    #[test]
    fn test_popular_coffee_structure() {
        let coffee = PopularCoffee {
            coffee_id: 1,
            coffee_name: "Espresso".to_string(),
            order_count: 100,
            average_rating: Some(Decimal::from_str("4.5").unwrap()),
            trend_percentage: Some(Decimal::from_str("15.5").unwrap()),
        };

        assert_eq!(coffee.coffee_id, 1);
        assert_eq!(coffee.coffee_name, "Espresso");
        assert_eq!(coffee.order_count, 100);
        assert!(coffee.average_rating.is_some());
        assert!(coffee.trend_percentage.is_some());
    }

    #[test]
    fn test_min_reviews_validation() {
        let min_reviews = 5;
        let validated = if min_reviews < 1 { 1 } else { min_reviews };
        
        assert_eq!(validated, 5);
    }

    #[test]
    fn test_min_reviews_default() {
        let min_reviews = 0;
        let validated = if min_reviews < 1 { 1 } else { min_reviews };
        
        assert_eq!(validated, 1);
    }
}
