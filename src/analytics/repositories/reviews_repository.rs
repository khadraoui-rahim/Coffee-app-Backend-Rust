// Reviews repository for analytics queries
// Provides aggregated data about reviews and ratings for analytics

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::analytics::types::{RatingDistribution, RatingTrend};

/// Repository for review analytics queries
#[derive(Clone)]
pub struct ReviewsAnalyticsRepository {
    pool: PgPool,
}

impl ReviewsAnalyticsRepository {
    /// Create a new ReviewsAnalyticsRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Calculate average rating across all coffees or for a specific coffee
    /// Returns the mean rating value
    pub async fn calculate_average_rating(
        &self,
        coffee_id: Option<i32>,
    ) -> Result<Option<Decimal>, sqlx::Error> {
        let result: (Option<Decimal>,) = match coffee_id {
            Some(id) => {
                sqlx::query_as(
                    r#"
                    SELECT AVG(rating)
                    FROM reviews
                    WHERE coffee_id = $1
                    "#
                )
                .bind(id)
                .fetch_one(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as(
                    r#"
                    SELECT AVG(rating)
                    FROM reviews
                    "#
                )
                .fetch_one(&self.pool)
                .await?
            }
        };

        Ok(result.0)
    }

    /// Get rating distribution grouped by rating value (1-5 stars)
    /// Returns count of reviews for each rating level
    pub async fn get_rating_distribution(
        &self,
        coffee_id: Option<i32>,
    ) -> Result<Vec<RatingDistribution>, sqlx::Error> {
        let results = match coffee_id {
            Some(id) => {
                sqlx::query_as::<_, (i32, i64)>(
                    r#"
                    SELECT rating, COUNT(*) as count
                    FROM reviews
                    WHERE coffee_id = $1
                    GROUP BY rating
                    ORDER BY rating ASC
                    "#
                )
                .bind(id)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, (i32, i64)>(
                    r#"
                    SELECT rating, COUNT(*) as count
                    FROM reviews
                    GROUP BY rating
                    ORDER BY rating ASC
                    "#
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(results
            .into_iter()
            .map(|(rating, count)| RatingDistribution { rating, count })
            .collect())
    }

    /// Get review trends over time as time-series data
    /// Returns average rating per day for trend analysis
    pub async fn get_review_trends(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        coffee_id: Option<i32>,
    ) -> Result<Vec<RatingTrend>, sqlx::Error> {
        let results = match coffee_id {
            Some(id) => {
                sqlx::query_as::<_, (DateTime<Utc>, Option<Decimal>)>(
                    r#"
                    SELECT 
                        DATE_TRUNC('day', created_at) as timestamp,
                        AVG(rating) as average_rating
                    FROM reviews
                    WHERE created_at >= $1 
                      AND created_at < $2
                      AND coffee_id = $3
                    GROUP BY timestamp
                    ORDER BY timestamp ASC
                    "#
                )
                .bind(start_date)
                .bind(end_date)
                .bind(id)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, (DateTime<Utc>, Option<Decimal>)>(
                    r#"
                    SELECT 
                        DATE_TRUNC('day', created_at) as timestamp,
                        AVG(rating) as average_rating
                    FROM reviews
                    WHERE created_at >= $1 
                      AND created_at < $2
                    GROUP BY timestamp
                    ORDER BY timestamp ASC
                    "#
                )
                .bind(start_date)
                .bind(end_date)
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(results
            .into_iter()
            .map(|(timestamp, average_rating)| RatingTrend {
                timestamp,
                average_rating: average_rating.unwrap_or(Decimal::ZERO),
            })
            .collect())
    }

    /// Count total reviews with optional coffee filter
    pub async fn count_reviews(
        &self,
        coffee_id: Option<i32>,
    ) -> Result<i64, sqlx::Error> {
        let result: (i64,) = match coffee_id {
            Some(id) => {
                sqlx::query_as(
                    r#"
                    SELECT COUNT(*)
                    FROM reviews
                    WHERE coffee_id = $1
                    "#
                )
                .bind(id)
                .fetch_one(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as(
                    r#"
                    SELECT COUNT(*)
                    FROM reviews
                    "#
                )
                .fetch_one(&self.pool)
                .await?
            }
        };

        Ok(result.0)
    }

    /// Get highest rated coffees with their average ratings
    pub async fn get_highest_rated_coffees(
        &self,
        limit: i32,
        min_reviews: i32,
    ) -> Result<Vec<(i32, String, Decimal, i64)>, sqlx::Error> {
        let results = sqlx::query_as::<_, (i32, String, Option<Decimal>, i64)>(
            r#"
            SELECT 
                c.id as coffee_id,
                c.name as coffee_name,
                AVG(r.rating) as average_rating,
                COUNT(r.id) as review_count
            FROM coffees c
            INNER JOIN reviews r ON c.id = r.coffee_id
            GROUP BY c.id, c.name
            HAVING COUNT(r.id) >= $1
            ORDER BY average_rating DESC, review_count DESC
            LIMIT $2
            "#
        )
        .bind(min_reviews as i64)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(results
            .into_iter()
            .map(|(coffee_id, coffee_name, average_rating, review_count)| {
                (
                    coffee_id,
                    coffee_name,
                    average_rating.unwrap_or(Decimal::ZERO),
                    review_count,
                )
            })
            .collect())
    }

    /// Get rating statistics for a specific time period
    pub async fn get_rating_statistics_by_period(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        coffee_id: Option<i32>,
    ) -> Result<(Option<Decimal>, i64), sqlx::Error> {
        let result: (Option<Decimal>, i64) = match coffee_id {
            Some(id) => {
                sqlx::query_as(
                    r#"
                    SELECT AVG(rating), COUNT(*)
                    FROM reviews
                    WHERE created_at >= $1 
                      AND created_at < $2
                      AND coffee_id = $3
                    "#
                )
                .bind(start_date)
                .bind(end_date)
                .bind(id)
                .fetch_one(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as(
                    r#"
                    SELECT AVG(rating), COUNT(*)
                    FROM reviews
                    WHERE created_at >= $1 
                      AND created_at < $2
                    "#
                )
                .bind(start_date)
                .bind(end_date)
                .fetch_one(&self.pool)
                .await?
            }
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_repository_creation() {
        let pool = PgPool::connect_lazy("postgresql://test").unwrap();
        let repo = ReviewsAnalyticsRepository::new(pool);
        assert!(std::mem::size_of_val(&repo) > 0);
    }

    #[test]
    fn test_rating_range_validation() {
        // Verify rating values are within valid range (1-5)
        let valid_ratings = vec![1, 2, 3, 4, 5];
        for rating in valid_ratings {
            assert!(rating >= 1 && rating <= 5);
        }
    }

    #[test]
    fn test_date_range_for_trends() {
        let now = Utc::now();
        let start = now - Duration::days(30);
        let end = now;
        
        assert!(start < end);
        assert_eq!((end - start).num_days(), 30);
    }

    #[test]
    fn test_rating_distribution_structure() {
        // Verify RatingDistribution structure
        let dist = RatingDistribution {
            rating: 5,
            count: 100,
        };
        assert_eq!(dist.rating, 5);
        assert_eq!(dist.count, 100);
    }

    #[test]
    fn test_rating_trend_structure() {
        // Verify RatingTrend structure
        let now = Utc::now();
        let trend = RatingTrend {
            timestamp: now,
            average_rating: Decimal::new(45, 1), // 4.5
        };
        assert_eq!(trend.timestamp, now);
        assert_eq!(trend.average_rating, Decimal::new(45, 1));
    }
}
