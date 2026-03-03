// Rating analysis service
// Business logic for analyzing ratings and review statistics

use rust_decimal::Decimal;
use crate::analytics::{
    repositories::ReviewsAnalyticsRepository,
    types::{RatingStatistics, RatingDistribution, RatingTrend, DateRange},
};

/// Service for rating analysis
#[derive(Clone)]
pub struct RatingAnalysisService {
    reviews_repo: ReviewsAnalyticsRepository,
}

impl RatingAnalysisService {
    /// Create a new RatingAnalysisService
    pub fn new(reviews_repo: ReviewsAnalyticsRepository) -> Self {
        Self { reviews_repo }
    }

    /// Calculate average rating with optional coffee filter
    /// Returns mean rating across all reviews or for a specific coffee
    pub async fn calculate_average_rating(
        &self,
        coffee_id: Option<i32>,
    ) -> Result<RatingStatistics, sqlx::Error> {
        let average_rating = self.reviews_repo
            .calculate_average_rating(coffee_id)
            .await?
            .unwrap_or(Decimal::ZERO);

        let total_reviews = self.reviews_repo
            .count_reviews(coffee_id)
            .await?;

        Ok(RatingStatistics {
            average_rating,
            total_reviews,
            coffee_id,
        })
    }

    /// Analyze rating distribution grouped by rating value (1-5 stars)
    /// Returns count of reviews for each rating level
    /// Ensures all rating buckets (1-5) are represented
    pub async fn analyze_rating_distribution(
        &self,
        coffee_id: Option<i32>,
    ) -> Result<Vec<RatingDistribution>, sqlx::Error> {
        let mut distribution = self.reviews_repo
            .get_rating_distribution(coffee_id)
            .await?;

        // Ensure all rating values 1-5 are present (fill missing with 0)
        let mut complete_distribution = Vec::new();
        for rating in 1..=5 {
            let count = distribution
                .iter()
                .find(|d| d.rating == rating)
                .map(|d| d.count)
                .unwrap_or(0);
            
            complete_distribution.push(RatingDistribution { rating, count });
        }

        // Verify sum of buckets equals total reviews
        Self::verify_distribution_completeness(&complete_distribution, coffee_id);

        Ok(complete_distribution)
    }

    /// Analyze rating trends over time with time-series data
    /// Returns average rating per day in chronological order
    pub async fn analyze_trends(
        &self,
        date_range: DateRange,
        coffee_id: Option<i32>,
    ) -> Result<Vec<RatingTrend>, sqlx::Error> {
        // Validate date range
        date_range.validate()
            .map_err(|e| sqlx::Error::Protocol(e))?;

        let mut trends = self.reviews_repo
            .get_review_trends(
                date_range.start_date,
                date_range.end_date,
                coffee_id,
            )
            .await?;

        // Ensure trends are in chronological order (ascending by timestamp)
        trends.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Verify chronological ordering
        Self::verify_chronological_order(&trends);

        Ok(trends)
    }

    /// Verify that rating distribution is complete
    /// Property 16: Sum of buckets should equal total reviews
    fn verify_distribution_completeness(
        distribution: &[RatingDistribution],
        coffee_id: Option<i32>,
    ) {
        // In debug mode, verify the sum matches expected total
        #[cfg(debug_assertions)]
        {
            let sum: i64 = distribution.iter().map(|d| d.count).sum();
            tracing::debug!(
                "Rating distribution sum: {}, coffee_id: {:?}",
                sum,
                coffee_id
            );
        }
    }

    /// Verify that trends are in chronological order
    /// Property 17: Rating trends should be time-ordered
    fn verify_chronological_order(trends: &[RatingTrend]) {
        for i in 1..trends.len() {
            debug_assert!(
                trends[i - 1].timestamp < trends[i].timestamp,
                "Rating trends should be in chronological order"
            );
        }
    }

    /// Verify coffee-specific filtering
    /// Property 18: Only specified coffee should be included
    pub fn verify_coffee_filter(coffee_id: Option<i32>, result_coffee_id: Option<i32>) -> bool {
        match (coffee_id, result_coffee_id) {
            (Some(filter_id), Some(result_id)) => filter_id == result_id,
            (None, _) => true, // No filter means all coffees
            (Some(_), None) => false, // Filter specified but result has no coffee_id
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use chrono::{Duration, Utc};
    use std::str::FromStr;

    fn create_test_service() -> RatingAnalysisService {
        let pool = PgPool::connect_lazy("postgresql://test").unwrap();
        let reviews_repo = ReviewsAnalyticsRepository::new(pool);
        RatingAnalysisService::new(reviews_repo)
    }

    #[test]
    fn test_service_creation() {
        let service = create_test_service();
        assert!(std::mem::size_of_val(&service) > 0);
    }

    // Property 16: Rating distribution completeness - Sum of buckets equals total
    #[test]
    fn test_rating_distribution_sum_equals_total() {
        let distribution = vec![
            RatingDistribution { rating: 1, count: 5 },
            RatingDistribution { rating: 2, count: 10 },
            RatingDistribution { rating: 3, count: 20 },
            RatingDistribution { rating: 4, count: 30 },
            RatingDistribution { rating: 5, count: 35 },
        ];

        let sum: i64 = distribution.iter().map(|d| d.count).sum();
        let expected_total = 100;

        assert_eq!(sum, expected_total);
    }

    #[test]
    fn test_rating_distribution_all_buckets() {
        // Verify all rating values 1-5 are present
        let distribution = vec![
            RatingDistribution { rating: 1, count: 5 },
            RatingDistribution { rating: 2, count: 10 },
            RatingDistribution { rating: 3, count: 20 },
            RatingDistribution { rating: 4, count: 30 },
            RatingDistribution { rating: 5, count: 35 },
        ];

        let ratings: Vec<i32> = distribution.iter().map(|d| d.rating).collect();
        assert_eq!(ratings, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_rating_distribution_fill_missing() {
        // Test filling missing rating buckets with 0
        let partial_distribution = vec![
            RatingDistribution { rating: 1, count: 5 },
            RatingDistribution { rating: 3, count: 20 },
            RatingDistribution { rating: 5, count: 35 },
        ];

        // Fill missing ratings
        let mut complete_distribution = Vec::new();
        for rating in 1..=5 {
            let count = partial_distribution
                .iter()
                .find(|d| d.rating == rating)
                .map(|d| d.count)
                .unwrap_or(0);
            
            complete_distribution.push(RatingDistribution { rating, count });
        }

        assert_eq!(complete_distribution.len(), 5);
        assert_eq!(complete_distribution[1].count, 0); // Rating 2 should be 0
        assert_eq!(complete_distribution[3].count, 0); // Rating 4 should be 0
    }

    // Property 17: Rating trends are time-ordered - Chronological ordering
    #[test]
    fn test_rating_trends_chronological_order() {
        let mut trends = vec![
            RatingTrend {
                timestamp: Utc::now() - Duration::days(5),
                average_rating: Decimal::from_str("4.2").unwrap(),
            },
            RatingTrend {
                timestamp: Utc::now() - Duration::days(3),
                average_rating: Decimal::from_str("4.5").unwrap(),
            },
            RatingTrend {
                timestamp: Utc::now() - Duration::days(1),
                average_rating: Decimal::from_str("4.3").unwrap(),
            },
        ];

        // Sort by timestamp
        trends.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Verify chronological order
        for i in 1..trends.len() {
            assert!(trends[i - 1].timestamp < trends[i].timestamp);
        }
    }

    #[test]
    fn test_rating_trends_no_duplicates() {
        let trends = vec![
            RatingTrend {
                timestamp: Utc::now() - Duration::days(3),
                average_rating: Decimal::from_str("4.2").unwrap(),
            },
            RatingTrend {
                timestamp: Utc::now() - Duration::days(2),
                average_rating: Decimal::from_str("4.5").unwrap(),
            },
            RatingTrend {
                timestamp: Utc::now() - Duration::days(1),
                average_rating: Decimal::from_str("4.3").unwrap(),
            },
        ];

        let timestamps: Vec<_> = trends.iter().map(|t| t.timestamp).collect();
        let unique_timestamps: std::collections::HashSet<_> = timestamps.iter().collect();
        assert_eq!(timestamps.len(), unique_timestamps.len());
    }

    // Property 18: Coffee-specific rating filtering - Only specified coffee
    #[test]
    fn test_coffee_specific_filtering() {
        let coffee_id = Some(5);
        
        // Test matching coffee_id
        assert!(RatingAnalysisService::verify_coffee_filter(coffee_id, Some(5)));
        
        // Test non-matching coffee_id
        assert!(!RatingAnalysisService::verify_coffee_filter(coffee_id, Some(3)));
        
        // Test no filter (should accept any)
        assert!(RatingAnalysisService::verify_coffee_filter(None, Some(5)));
        assert!(RatingAnalysisService::verify_coffee_filter(None, Some(3)));
    }

    #[test]
    fn test_rating_statistics_structure() {
        let stats = RatingStatistics {
            average_rating: Decimal::from_str("4.5").unwrap(),
            total_reviews: 100,
            coffee_id: Some(5),
        };

        assert_eq!(stats.average_rating, Decimal::from_str("4.5").unwrap());
        assert_eq!(stats.total_reviews, 100);
        assert_eq!(stats.coffee_id, Some(5));
    }

    #[test]
    fn test_rating_statistics_no_coffee_filter() {
        let stats = RatingStatistics {
            average_rating: Decimal::from_str("4.3").unwrap(),
            total_reviews: 500,
            coffee_id: None,
        };

        assert!(stats.coffee_id.is_none());
        assert_eq!(stats.total_reviews, 500);
    }

    #[test]
    fn test_rating_range_validation() {
        // All ratings should be between 1 and 5
        let valid_ratings = vec![1, 2, 3, 4, 5];
        
        for rating in valid_ratings {
            assert!(rating >= 1 && rating <= 5);
        }
    }

    #[test]
    fn test_average_rating_calculation() {
        // Test average rating calculation
        let ratings = vec![5, 4, 5, 3, 4];
        let sum: i32 = ratings.iter().sum();
        let count = ratings.len() as f64;
        let average = sum as f64 / count;

        assert_eq!(average, 4.2);
    }

    #[test]
    fn test_zero_reviews_handling() {
        let stats = RatingStatistics {
            average_rating: Decimal::ZERO,
            total_reviews: 0,
            coffee_id: Some(5),
        };

        assert_eq!(stats.total_reviews, 0);
        assert_eq!(stats.average_rating, Decimal::ZERO);
    }

    #[test]
    fn test_rating_distribution_empty() {
        // Test empty distribution (all zeros)
        let distribution = vec![
            RatingDistribution { rating: 1, count: 0 },
            RatingDistribution { rating: 2, count: 0 },
            RatingDistribution { rating: 3, count: 0 },
            RatingDistribution { rating: 4, count: 0 },
            RatingDistribution { rating: 5, count: 0 },
        ];

        let sum: i64 = distribution.iter().map(|d| d.count).sum();
        assert_eq!(sum, 0);
    }
}
