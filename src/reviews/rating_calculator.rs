use crate::reviews::{ReviewRepository, ServiceError};

/// Calculator for computing and updating average ratings
#[derive(Clone)]
pub struct RatingCalculator {
    repository: ReviewRepository,
}

impl RatingCalculator {
    /// Create a new RatingCalculator
    pub fn new(repository: ReviewRepository) -> Self {
        Self { repository }
    }

    /// Recalculate and update the average rating for a coffee
    /// 
    /// This method:
    /// 1. Fetches all ratings for the given coffee
    /// 2. Calculates the arithmetic mean
    /// 3. Updates the coffees table with the new average and count
    /// 4. Returns the calculated average (or None if no reviews exist)
    pub async fn recalculate_average(&self, coffee_id: i32) -> Result<Option<f64>, ServiceError> {
        // Get all ratings for this coffee
        let ratings = self.repository.get_ratings_for_coffee(coffee_id).await?;

        // Calculate average and count
        let count = ratings.len() as i32;
        let average = if ratings.is_empty() {
            None
        } else {
            let sum: i32 = ratings.iter().map(|&r| r as i32).sum();
            let avg = sum as f64 / ratings.len() as f64;
            Some(avg)
        };

        // Update the coffees table
        self.repository
            .update_coffee_rating(coffee_id, average, count)
            .await?;

        Ok(average)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Helper function to create a test database pool
    async fn create_test_pool() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| {
                "postgresql://coffee_user:coffee_pass@test_db:5432/coffee_test_db".to_string()
            });
        
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database");
        
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");
        
        pool
    }

    /// Helper function to create a test user with unique email
    async fn create_test_user(pool: &PgPool) -> i32 {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let email = format!("calc{}@example.com", timestamp);
        
        let user_id: (i32,) = sqlx::query_as(
            "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING id"
        )
        .bind(email)
        .bind("test_hash")
        .fetch_one(pool)
        .await
        .expect("Failed to create test user");
        
        user_id.0
    }

    /// Helper function to create a test coffee with unique name
    async fn create_test_coffee(pool: &PgPool) -> i32 {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let name = format!("Calc Coffee {}", timestamp);
        
        let coffee_id: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO coffees (image_url, name, coffee_type, price, rating)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#
        )
        .bind("https://test.com/image.jpg")
        .bind(name)
        .bind("Test Type")
        .bind(3.50)
        .bind(4.5)
        .fetch_one(pool)
        .await
        .expect("Failed to create test coffee");
        
        coffee_id.0
    }

    #[tokio::test]
    async fn test_recalculate_average_with_reviews() {
        let pool = create_test_pool().await;
        let user1 = create_test_user(&pool).await;
        let user2 = create_test_user(&pool).await;
        let user3 = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;
        
        let repository = ReviewRepository::new(pool.clone());
        
        // Create reviews with ratings: 5, 4, 3
        repository.create(user1, coffee_id, 5, None).await.unwrap();
        repository.create(user2, coffee_id, 4, None).await.unwrap();
        repository.create(user3, coffee_id, 3, None).await.unwrap();
        
        let calculator = RatingCalculator::new(repository);
        
        // Recalculate average
        let average = calculator
            .recalculate_average(coffee_id)
            .await
            .expect("Failed to recalculate average");
        
        // Average should be (5 + 4 + 3) / 3 = 4.0
        assert_eq!(average, Some(4.0));
        
        // Verify the database was updated
        let count: i32 = sqlx::query_scalar(
            "SELECT review_count FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch review count");
        
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_recalculate_average_no_reviews() {
        let pool = create_test_pool().await;
        let coffee_id = create_test_coffee(&pool).await;
        
        let repository = ReviewRepository::new(pool.clone());
        let calculator = RatingCalculator::new(repository);
        
        // Recalculate average with no reviews
        let average = calculator
            .recalculate_average(coffee_id)
            .await
            .expect("Failed to recalculate average");
        
        // Average should be None when there are no reviews
        assert_eq!(average, None);
        
        // Verify the database was updated with count 0
        let count: i32 = sqlx::query_scalar(
            "SELECT review_count FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch review count");
        
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_recalculate_average_single_review() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;
        
        let repository = ReviewRepository::new(pool.clone());
        
        // Create single review with rating 5
        repository.create(user_id, coffee_id, 5, None).await.unwrap();
        
        let calculator = RatingCalculator::new(repository);
        
        // Recalculate average
        let average = calculator
            .recalculate_average(coffee_id)
            .await
            .expect("Failed to recalculate average");
        
        // Average should be 5.0
        assert_eq!(average, Some(5.0));
        
        // Verify count is 1
        let count: i32 = sqlx::query_scalar(
            "SELECT review_count FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch review count");
        
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_recalculate_average_all_same_rating() {
        let pool = create_test_pool().await;
        let user1 = create_test_user(&pool).await;
        let user2 = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;
        
        let repository = ReviewRepository::new(pool.clone());
        
        // Create reviews with same rating: 4, 4
        repository.create(user1, coffee_id, 4, None).await.unwrap();
        repository.create(user2, coffee_id, 4, None).await.unwrap();
        
        let calculator = RatingCalculator::new(repository);
        
        // Recalculate average
        let average = calculator
            .recalculate_average(coffee_id)
            .await
            .expect("Failed to recalculate average");
        
        // Average should be 4.0
        assert_eq!(average, Some(4.0));
    }

    #[tokio::test]
    async fn test_recalculate_average_decimal_result() {
        let pool = create_test_pool().await;
        let user1 = create_test_user(&pool).await;
        let user2 = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;
        
        let repository = ReviewRepository::new(pool.clone());
        
        // Create reviews with ratings: 5, 4
        repository.create(user1, coffee_id, 5, None).await.unwrap();
        repository.create(user2, coffee_id, 4, None).await.unwrap();
        
        let calculator = RatingCalculator::new(repository);
        
        // Recalculate average
        let average = calculator
            .recalculate_average(coffee_id)
            .await
            .expect("Failed to recalculate average");
        
        // Average should be (5 + 4) / 2 = 4.5
        assert_eq!(average, Some(4.5));
    }
}
