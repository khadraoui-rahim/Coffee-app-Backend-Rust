use crate::reviews::{
    CreateReviewRequest, Review, ReviewRepository, RatingCalculator, ServiceError,
    UpdateReviewRequest,
};
use validator::Validate;

/// Service layer for review business logic
#[derive(Clone)]
pub struct ReviewService {
    repository: ReviewRepository,
    rating_calculator: RatingCalculator,
}

impl ReviewService {
    /// Create a new ReviewService
    pub fn new(repository: ReviewRepository, rating_calculator: RatingCalculator) -> Self {
        Self {
            repository,
            rating_calculator,
        }
    }

    /// Create a new review
    ///
    /// This method:
    /// 1. Validates the request
    /// 2. Checks for duplicate reviews (user already reviewed this coffee)
    /// 3. Verifies the coffee exists
    /// 4. Creates the review
    /// 5. Recalculates the average rating for the coffee
    pub async fn create_review(
        &self,
        user_id: i32,
        request: CreateReviewRequest,
    ) -> Result<Review, ServiceError> {
        // 1. Validate request
        request
            .validate()
            .map_err(|e| ServiceError::ValidationError(format!("Validation failed: {}", e)))?;

        // 2. Check for duplicate review
        if let Some(_existing) = self
            .repository
            .find_by_user_and_coffee(user_id, request.coffee_id)
            .await?
        {
            return Err(ServiceError::DuplicateReview);
        }

        // 3. Verify coffee exists
        if !self.repository.coffee_exists(request.coffee_id).await? {
            return Err(ServiceError::CoffeeNotFound);
        }

        // 4. Create the review
        let review = self
            .repository
            .create(user_id, request.coffee_id, request.rating, request.comment)
            .await?;

        // 5. Recalculate average rating
        self.rating_calculator
            .recalculate_average(request.coffee_id)
            .await?;

        Ok(review)
    }

    /// Update an existing review
    ///
    /// This method:
    /// 1. Validates the request
    /// 2. Fetches the existing review
    /// 3. Verifies the user owns the review
    /// 4. Updates the review
    /// 5. Recalculates the average rating if the rating changed
    pub async fn update_review(
        &self,
        review_id: i32,
        user_id: i32,
        request: UpdateReviewRequest,
    ) -> Result<Review, ServiceError> {
        // 1. Validate request
        request
            .validate()
            .map_err(|e| ServiceError::ValidationError(format!("Validation failed: {}", e)))?;

        // 2. Fetch existing review
        let existing = self
            .repository
            .find_by_id(review_id)
            .await?
            .ok_or(ServiceError::NotFound)?;

        // 3. Verify ownership
        if existing.user_id != user_id {
            return Err(ServiceError::Unauthorized);
        }

        // 4. Update the review
        let updated = self
            .repository
            .update(review_id, request.rating, request.comment)
            .await?;

        // 5. Recalculate average rating if rating changed
        if request.rating.is_some() && request.rating != Some(existing.rating) {
            self.rating_calculator
                .recalculate_average(existing.coffee_id)
                .await?;
        }

        Ok(updated)
    }

    /// Delete a review
    ///
    /// This method:
    /// 1. Fetches the existing review
    /// 2. Verifies the user owns the review
    /// 3. Deletes the review
    /// 4. Recalculates the average rating
    pub async fn delete_review(&self, review_id: i32, user_id: i32) -> Result<(), ServiceError> {
        // 1. Fetch existing review
        let existing = self
            .repository
            .find_by_id(review_id)
            .await?
            .ok_or(ServiceError::NotFound)?;

        // 2. Verify ownership
        if existing.user_id != user_id {
            return Err(ServiceError::Unauthorized);
        }

        let coffee_id = existing.coffee_id;

        // 3. Delete the review
        self.repository.delete(review_id).await?;

        // 4. Recalculate average rating
        self.rating_calculator
            .recalculate_average(coffee_id)
            .await?;

        Ok(())
    }

    /// Get all reviews for a coffee
    pub async fn get_reviews_for_coffee(&self, coffee_id: i32) -> Result<Vec<Review>, ServiceError> {
        self.repository.find_by_coffee(coffee_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Helper function to create a test database pool
    async fn create_test_pool() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
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
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
        let email = format!("svc{}{}@example.com", timestamp, counter);
        
        let user_id: (i32,) = sqlx::query_as(
            "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING id",
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
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
        let name = format!("Svc Coffee {}{}", timestamp, counter);
        
        let coffee_id: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO coffees (image_url, name, coffee_type, price, rating)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
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

    /// Helper function to create a service
    fn create_service(pool: PgPool) -> ReviewService {
        let repository = ReviewRepository::new(pool);
        let rating_calculator = RatingCalculator::new(repository.clone());
        ReviewService::new(repository, rating_calculator)
    }

    #[tokio::test]
    async fn test_create_review_success() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        let request = CreateReviewRequest {
            coffee_id,
            rating: 5,
            comment: Some("Excellent!".to_string()),
        };

        let review = service
            .create_review(user_id, request)
            .await
            .expect("Failed to create review");

        assert_eq!(review.user_id, user_id);
        assert_eq!(review.coffee_id, coffee_id);
        assert_eq!(review.rating, 5);
        assert_eq!(review.comment, Some("Excellent!".to_string()));
    }

    #[tokio::test]
    async fn test_create_review_duplicate() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        let request = CreateReviewRequest {
            coffee_id,
            rating: 5,
            comment: None,
        };

        // Create first review
        service
            .create_review(user_id, request.clone())
            .await
            .expect("Failed to create first review");

        // Try to create duplicate
        let result = service.create_review(user_id, request).await;

        assert!(result.is_err());
        match result {
            Err(ServiceError::DuplicateReview) => (),
            _ => panic!("Expected DuplicateReview error"),
        }
    }

    #[tokio::test]
    async fn test_create_review_coffee_not_found() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;

        let service = create_service(pool.clone());

        let request = CreateReviewRequest {
            coffee_id: 99999,
            rating: 5,
            comment: None,
        };

        let result = service.create_review(user_id, request).await;

        assert!(result.is_err());
        match result {
            Err(ServiceError::CoffeeNotFound) => (),
            _ => panic!("Expected CoffeeNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_create_review_invalid_rating() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        let request = CreateReviewRequest {
            coffee_id,
            rating: 6, // Invalid: must be 1-5
            comment: None,
        };

        let result = service.create_review(user_id, request).await;

        assert!(result.is_err());
        match result {
            Err(ServiceError::ValidationError(_)) => (),
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_update_review_success() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create review
        let create_request = CreateReviewRequest {
            coffee_id,
            rating: 3,
            comment: Some("OK".to_string()),
        };

        let review = service
            .create_review(user_id, create_request)
            .await
            .expect("Failed to create review");

        // Update review
        let update_request = UpdateReviewRequest {
            rating: Some(5),
            comment: Some("Excellent!".to_string()),
        };

        let updated = service
            .update_review(review.id, user_id, update_request)
            .await
            .expect("Failed to update review");

        assert_eq!(updated.rating, 5);
        assert_eq!(updated.comment, Some("Excellent!".to_string()));
    }

    #[tokio::test]
    async fn test_update_review_unauthorized() {
        let pool = create_test_pool().await;
        let user1 = create_test_user(&pool).await;
        let user2 = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // User1 creates review
        let create_request = CreateReviewRequest {
            coffee_id,
            rating: 5,
            comment: None,
        };

        let review = service
            .create_review(user1, create_request)
            .await
            .expect("Failed to create review");

        // User2 tries to update user1's review
        let update_request = UpdateReviewRequest {
            rating: Some(1),
            comment: None,
        };

        let result = service.update_review(review.id, user2, update_request).await;

        assert!(result.is_err());
        match result {
            Err(ServiceError::Unauthorized) => (),
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[tokio::test]
    async fn test_update_review_not_found() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;

        let service = create_service(pool.clone());

        let update_request = UpdateReviewRequest {
            rating: Some(5),
            comment: None,
        };

        let result = service.update_review(99999, user_id, update_request).await;

        assert!(result.is_err());
        match result {
            Err(ServiceError::NotFound) => (),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_delete_review_success() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create review
        let create_request = CreateReviewRequest {
            coffee_id,
            rating: 5,
            comment: None,
        };

        let review = service
            .create_review(user_id, create_request)
            .await
            .expect("Failed to create review");

        // Delete review
        service
            .delete_review(review.id, user_id)
            .await
            .expect("Failed to delete review");

        // Verify it's deleted
        let repository = ReviewRepository::new(pool);
        let result = repository.find_by_id(review.id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_review_unauthorized() {
        let pool = create_test_pool().await;
        let user1 = create_test_user(&pool).await;
        let user2 = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // User1 creates review
        let create_request = CreateReviewRequest {
            coffee_id,
            rating: 5,
            comment: None,
        };

        let review = service
            .create_review(user1, create_request)
            .await
            .expect("Failed to create review");

        // User2 tries to delete user1's review
        let result = service.delete_review(review.id, user2).await;

        assert!(result.is_err());
        match result {
            Err(ServiceError::Unauthorized) => (),
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[tokio::test]
    async fn test_delete_review_not_found() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;

        let service = create_service(pool.clone());

        let result = service.delete_review(99999, user_id).await;

        assert!(result.is_err());
        match result {
            Err(ServiceError::NotFound) => (),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_reviews_for_coffee() {
        let pool = create_test_pool().await;
        let user1 = create_test_user(&pool).await;
        let user2 = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create two reviews
        service
            .create_review(
                user1,
                CreateReviewRequest {
                    coffee_id,
                    rating: 5,
                    comment: None,
                },
            )
            .await
            .unwrap();

        service
            .create_review(
                user2,
                CreateReviewRequest {
                    coffee_id,
                    rating: 4,
                    comment: None,
                },
            )
            .await
            .unwrap();

        // Get all reviews
        let reviews = service
            .get_reviews_for_coffee(coffee_id)
            .await
            .expect("Failed to get reviews");

        assert_eq!(reviews.len(), 2);
    }

    // ============================================================================
    // Validation Tests (Task 9)
    // ============================================================================

    // Property 17: Rating Range Validation
    // Validates: Requirements 8.1, 8.2
    #[tokio::test]
    async fn test_rating_below_minimum_rejected() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        let request = CreateReviewRequest {
            coffee_id,
            rating: 0, // Below minimum
            comment: None,
        };

        let result = service.create_review(user_id, request).await;

        assert!(result.is_err());
        match result {
            Err(ServiceError::ValidationError(_)) => (),
            _ => panic!("Expected ValidationError for rating below minimum"),
        }
    }

    #[tokio::test]
    async fn test_rating_above_maximum_rejected() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        let request = CreateReviewRequest {
            coffee_id,
            rating: 6, // Above maximum
            comment: None,
        };

        let result = service.create_review(user_id, request).await;

        assert!(result.is_err());
        match result {
            Err(ServiceError::ValidationError(_)) => (),
            _ => panic!("Expected ValidationError for rating above maximum"),
        }
    }

    #[tokio::test]
    async fn test_rating_at_minimum_accepted() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        let request = CreateReviewRequest {
            coffee_id,
            rating: 1, // Minimum valid rating
            comment: None,
        };

        let result = service.create_review(user_id, request).await;

        assert!(result.is_ok());
        let review = result.unwrap();
        assert_eq!(review.rating, 1);
    }

    #[tokio::test]
    async fn test_rating_at_maximum_accepted() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        let request = CreateReviewRequest {
            coffee_id,
            rating: 5, // Maximum valid rating
            comment: None,
        };

        let result = service.create_review(user_id, request).await;

        assert!(result.is_ok());
        let review = result.unwrap();
        assert_eq!(review.rating, 5);
    }

    #[tokio::test]
    async fn test_update_rating_below_minimum_rejected() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create a valid review first
        let create_request = CreateReviewRequest {
            coffee_id,
            rating: 3,
            comment: None,
        };

        let review = service
            .create_review(user_id, create_request)
            .await
            .unwrap();

        // Try to update with invalid rating
        let update_request = UpdateReviewRequest {
            rating: Some(0), // Below minimum
            comment: None,
        };

        let result = service.update_review(review.id, user_id, update_request).await;

        assert!(result.is_err());
        match result {
            Err(ServiceError::ValidationError(_)) => (),
            _ => panic!("Expected ValidationError for rating below minimum"),
        }
    }

    #[tokio::test]
    async fn test_update_rating_above_maximum_rejected() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create a valid review first
        let create_request = CreateReviewRequest {
            coffee_id,
            rating: 3,
            comment: None,
        };

        let review = service
            .create_review(user_id, create_request)
            .await
            .unwrap();

        // Try to update with invalid rating
        let update_request = UpdateReviewRequest {
            rating: Some(6), // Above maximum
            comment: None,
        };

        let result = service.update_review(review.id, user_id, update_request).await;

        assert!(result.is_err());
        match result {
            Err(ServiceError::ValidationError(_)) => (),
            _ => panic!("Expected ValidationError for rating above maximum"),
        }
    }

    // Property 18: Comment Length Validation
    // Validates: Requirements 8.3
    #[tokio::test]
    async fn test_comment_exceeding_max_length_rejected() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create a comment that exceeds 1000 characters
        let long_comment = "a".repeat(1001);

        let request = CreateReviewRequest {
            coffee_id,
            rating: 5,
            comment: Some(long_comment),
        };

        let result = service.create_review(user_id, request).await;

        assert!(result.is_err());
        match result {
            Err(ServiceError::ValidationError(_)) => (),
            _ => panic!("Expected ValidationError for comment exceeding max length"),
        }
    }

    #[tokio::test]
    async fn test_comment_at_max_length_accepted() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create a comment exactly at 1000 characters
        let max_comment = "a".repeat(1000);

        let request = CreateReviewRequest {
            coffee_id,
            rating: 5,
            comment: Some(max_comment.clone()),
        };

        let result = service.create_review(user_id, request).await;

        assert!(result.is_ok());
        let review = result.unwrap();
        assert_eq!(review.comment, Some(max_comment));
    }

    #[tokio::test]
    async fn test_empty_comment_accepted() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        let request = CreateReviewRequest {
            coffee_id,
            rating: 5,
            comment: Some(String::new()), // Empty string
        };

        let result = service.create_review(user_id, request).await;

        assert!(result.is_ok());
        let review = result.unwrap();
        assert_eq!(review.comment, Some(String::new()));
    }

    #[tokio::test]
    async fn test_none_comment_accepted() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        let request = CreateReviewRequest {
            coffee_id,
            rating: 5,
            comment: None, // No comment
        };

        let result = service.create_review(user_id, request).await;

        assert!(result.is_ok());
        let review = result.unwrap();
        assert_eq!(review.comment, None);
    }

    #[tokio::test]
    async fn test_update_comment_exceeding_max_length_rejected() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create a valid review first
        let create_request = CreateReviewRequest {
            coffee_id,
            rating: 3,
            comment: Some("Initial comment".to_string()),
        };

        let review = service
            .create_review(user_id, create_request)
            .await
            .unwrap();

        // Try to update with comment exceeding max length
        let long_comment = "a".repeat(1001);
        let update_request = UpdateReviewRequest {
            rating: None,
            comment: Some(long_comment),
        };

        let result = service.update_review(review.id, user_id, update_request).await;

        assert!(result.is_err());
        match result {
            Err(ServiceError::ValidationError(_)) => (),
            _ => panic!("Expected ValidationError for comment exceeding max length"),
        }
    }

    // Property 19: Required Fields Validation
    // Validates: Requirements 8.5
    #[tokio::test]
    async fn test_create_review_with_all_required_fields() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // All required fields present (coffee_id and rating)
        let request = CreateReviewRequest {
            coffee_id,
            rating: 4,
            comment: None,
        };

        let result = service.create_review(user_id, request).await;

        assert!(result.is_ok());
        let review = result.unwrap();
        assert_eq!(review.coffee_id, coffee_id);
        assert_eq!(review.rating, 4);
    }

    #[tokio::test]
    async fn test_valid_ratings_in_range() {
        let pool = create_test_pool().await;
        let coffee_id = create_test_coffee(&pool).await;
        let service = create_service(pool.clone());

        // Test all valid ratings (1-5)
        for rating in 1..=5 {
            let user_id = create_test_user(&pool).await;
            let request = CreateReviewRequest {
                coffee_id,
                rating,
                comment: None,
            };

            let result = service.create_review(user_id, request).await;
            assert!(result.is_ok(), "Rating {} should be valid", rating);
            let review = result.unwrap();
            assert_eq!(review.rating, rating);
        }
    }

    // ============================================================================
    // Timestamp Tests (Task 10)
    // ============================================================================

    // Property 2: Review Creation Timestamp
    // Validates: Requirements 1.4
    #[tokio::test]
    async fn test_creation_timestamp_is_set() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        let before_creation = chrono::Utc::now();

        let request = CreateReviewRequest {
            coffee_id,
            rating: 5,
            comment: Some("Test comment".to_string()),
        };

        let review = service
            .create_review(user_id, request)
            .await
            .expect("Failed to create review");

        let after_creation = chrono::Utc::now();

        // Verify created_at is set and within reasonable bounds
        assert!(
            review.created_at >= before_creation,
            "created_at should be after or equal to before_creation"
        );
        assert!(
            review.created_at <= after_creation,
            "created_at should be before or equal to after_creation"
        );

        // Verify created_at and updated_at are initially the same
        assert_eq!(
            review.created_at, review.updated_at,
            "created_at and updated_at should be equal on creation"
        );
    }

    #[tokio::test]
    async fn test_creation_timestamp_is_recent() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        let request = CreateReviewRequest {
            coffee_id,
            rating: 4,
            comment: None,
        };

        let review = service
            .create_review(user_id, request)
            .await
            .expect("Failed to create review");

        let now = chrono::Utc::now();
        let time_diff = now - review.created_at;

        // Verify timestamp is recent (within 5 seconds)
        assert!(
            time_diff.num_seconds() < 5,
            "created_at should be within 5 seconds of now"
        );
    }

    // Property 7: Update Timestamp Advancement
    // Validates: Requirements 3.3
    #[tokio::test]
    async fn test_update_timestamp_advances() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create a review
        let create_request = CreateReviewRequest {
            coffee_id,
            rating: 3,
            comment: Some("Initial comment".to_string()),
        };

        let created_review = service
            .create_review(user_id, create_request)
            .await
            .expect("Failed to create review");

        // Wait a small amount to ensure timestamp difference
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Update the review
        let update_request = UpdateReviewRequest {
            rating: Some(5),
            comment: Some("Updated comment".to_string()),
        };

        let updated_review = service
            .update_review(created_review.id, user_id, update_request)
            .await
            .expect("Failed to update review");

        // Verify updated_at has advanced
        assert!(
            updated_review.updated_at > created_review.updated_at,
            "updated_at should be greater than the original updated_at"
        );

        // Verify created_at remains unchanged
        assert_eq!(
            updated_review.created_at, created_review.created_at,
            "created_at should not change on update"
        );
    }

    #[tokio::test]
    async fn test_update_timestamp_is_recent() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create a review
        let create_request = CreateReviewRequest {
            coffee_id,
            rating: 3,
            comment: Some("Initial".to_string()),
        };

        let created_review = service
            .create_review(user_id, create_request)
            .await
            .expect("Failed to create review");

        // Wait a bit
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let before_update = chrono::Utc::now();

        // Update the review
        let update_request = UpdateReviewRequest {
            rating: Some(4),
            comment: None,
        };

        let updated_review = service
            .update_review(created_review.id, user_id, update_request)
            .await
            .expect("Failed to update review");

        let after_update = chrono::Utc::now();

        // Verify updated_at is within reasonable bounds
        assert!(
            updated_review.updated_at >= before_update,
            "updated_at should be after or equal to before_update"
        );
        assert!(
            updated_review.updated_at <= after_update,
            "updated_at should be before or equal to after_update"
        );
    }

    #[tokio::test]
    async fn test_multiple_updates_advance_timestamp() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create a review
        let create_request = CreateReviewRequest {
            coffee_id,
            rating: 3,
            comment: Some("Initial".to_string()),
        };

        let mut previous_review = service
            .create_review(user_id, create_request)
            .await
            .expect("Failed to create review");

        // Perform multiple updates
        for i in 1..=3 {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            let update_request = UpdateReviewRequest {
                rating: Some(i + 2),
                comment: Some(format!("Update {}", i)),
            };

            let updated_review = service
                .update_review(previous_review.id, user_id, update_request)
                .await
                .expect("Failed to update review");

            // Verify updated_at advances with each update
            assert!(
                updated_review.updated_at > previous_review.updated_at,
                "updated_at should advance on update {}", i
            );

            // Verify created_at never changes
            assert_eq!(
                updated_review.created_at, previous_review.created_at,
                "created_at should remain constant on update {}", i
            );

            previous_review = updated_review;
        }
    }

    #[tokio::test]
    async fn test_update_comment_only_advances_timestamp() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create a review
        let create_request = CreateReviewRequest {
            coffee_id,
            rating: 4,
            comment: Some("Initial".to_string()),
        };

        let created_review = service
            .create_review(user_id, create_request)
            .await
            .expect("Failed to create review");

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Update only the comment
        let update_request = UpdateReviewRequest {
            rating: None,
            comment: Some("Updated comment only".to_string()),
        };

        let updated_review = service
            .update_review(created_review.id, user_id, update_request)
            .await
            .expect("Failed to update review");

        // Verify updated_at advances even when only comment is updated
        assert!(
            updated_review.updated_at > created_review.updated_at,
            "updated_at should advance when only comment is updated"
        );

        // Verify rating didn't change
        assert_eq!(
            updated_review.rating, created_review.rating,
            "rating should remain unchanged"
        );
    }

    #[tokio::test]
    async fn test_update_rating_only_advances_timestamp() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create a review
        let create_request = CreateReviewRequest {
            coffee_id,
            rating: 3,
            comment: Some("Comment".to_string()),
        };

        let created_review = service
            .create_review(user_id, create_request)
            .await
            .expect("Failed to create review");

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Update only the rating
        let update_request = UpdateReviewRequest {
            rating: Some(5),
            comment: None,
        };

        let updated_review = service
            .update_review(created_review.id, user_id, update_request)
            .await
            .expect("Failed to update review");

        // Verify updated_at advances even when only rating is updated
        assert!(
            updated_review.updated_at > created_review.updated_at,
            "updated_at should advance when only rating is updated"
        );

        // Verify comment didn't change
        assert_eq!(
            updated_review.comment, created_review.comment,
            "comment should remain unchanged"
        );
    }

    // ============================================================================
    // Rating Recalculation Tests (Task 11)
    // ============================================================================

    // Property 15: Average Rating After Update
    // Validates: Requirements 5.2
    #[tokio::test]
    async fn test_average_rating_recalculated_after_update() {
        let pool = create_test_pool().await;
        let user1 = create_test_user(&pool).await;
        let user2 = create_test_user(&pool).await;
        let user3 = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create three reviews: ratings 5, 4, 3
        // Initial average: (5 + 4 + 3) / 3 = 4.0
        service
            .create_review(
                user1,
                CreateReviewRequest {
                    coffee_id,
                    rating: 5,
                    comment: None,
                },
            )
            .await
            .unwrap();

        let review2 = service
            .create_review(
                user2,
                CreateReviewRequest {
                    coffee_id,
                    rating: 4,
                    comment: None,
                },
            )
            .await
            .unwrap();

        service
            .create_review(
                user3,
                CreateReviewRequest {
                    coffee_id,
                    rating: 3,
                    comment: None,
                },
            )
            .await
            .unwrap();

        // Verify initial average is 4.0
        let initial_avg: Option<f64> = sqlx::query_scalar(
            "SELECT average_rating::float8 FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch initial average");

        assert_eq!(initial_avg, Some(4.0), "Initial average should be 4.0");

        // Update user2's review from 4 to 1
        // New average: (5 + 1 + 3) / 3 = 3.0
        service
            .update_review(
                review2.id,
                user2,
                UpdateReviewRequest {
                    rating: Some(1),
                    comment: None,
                },
            )
            .await
            .expect("Failed to update review");

        // Verify average was recalculated to 3.0
        let updated_avg: Option<f64> = sqlx::query_scalar(
            "SELECT average_rating::float8 FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch updated average");

        assert_eq!(updated_avg, Some(3.0), "Updated average should be 3.0");

        // Verify review count remains 3
        let count: i32 = sqlx::query_scalar(
            "SELECT review_count FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch review count");

        assert_eq!(count, 3, "Review count should remain 3");
    }

    #[tokio::test]
    async fn test_average_rating_unchanged_when_comment_only_updated() {
        let pool = create_test_pool().await;
        let user1 = create_test_user(&pool).await;
        let user2 = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create two reviews: ratings 5, 3
        // Average: (5 + 3) / 2 = 4.0
        service
            .create_review(
                user1,
                CreateReviewRequest {
                    coffee_id,
                    rating: 5,
                    comment: None,
                },
            )
            .await
            .unwrap();

        let review2 = service
            .create_review(
                user2,
                CreateReviewRequest {
                    coffee_id,
                    rating: 3,
                    comment: Some("Initial comment".to_string()),
                },
            )
            .await
            .unwrap();

        // Get initial average
        let initial_avg: Option<f64> = sqlx::query_scalar(
            "SELECT average_rating::float8 FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch initial average");

        assert_eq!(initial_avg, Some(4.0), "Initial average should be 4.0");

        // Update only the comment (not the rating)
        service
            .update_review(
                review2.id,
                user2,
                UpdateReviewRequest {
                    rating: None,
                    comment: Some("Updated comment".to_string()),
                },
            )
            .await
            .expect("Failed to update review");

        // Verify average remains unchanged
        let updated_avg: Option<f64> = sqlx::query_scalar(
            "SELECT average_rating::float8 FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch updated average");

        assert_eq!(updated_avg, Some(4.0), "Average should remain 4.0 when only comment is updated");
    }

    #[tokio::test]
    async fn test_average_rating_recalculated_to_higher_value() {
        let pool = create_test_pool().await;
        let user1 = create_test_user(&pool).await;
        let user2 = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create two reviews: ratings 2, 3
        // Average: (2 + 3) / 2 = 2.5
        let review1 = service
            .create_review(
                user1,
                CreateReviewRequest {
                    coffee_id,
                    rating: 2,
                    comment: None,
                },
            )
            .await
            .unwrap();

        service
            .create_review(
                user2,
                CreateReviewRequest {
                    coffee_id,
                    rating: 3,
                    comment: None,
                },
            )
            .await
            .unwrap();

        // Update user1's review from 2 to 5
        // New average: (5 + 3) / 2 = 4.0
        service
            .update_review(
                review1.id,
                user1,
                UpdateReviewRequest {
                    rating: Some(5),
                    comment: None,
                },
            )
            .await
            .expect("Failed to update review");

        // Verify average increased to 4.0
        let updated_avg: Option<f64> = sqlx::query_scalar(
            "SELECT average_rating::float8 FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch updated average");

        assert_eq!(updated_avg, Some(4.0), "Average should increase to 4.0");
    }

    #[tokio::test]
    async fn test_average_rating_recalculated_for_single_review_update() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create single review with rating 3
        let review = service
            .create_review(
                user_id,
                CreateReviewRequest {
                    coffee_id,
                    rating: 3,
                    comment: None,
                },
            )
            .await
            .unwrap();

        // Update to rating 5
        service
            .update_review(
                review.id,
                user_id,
                UpdateReviewRequest {
                    rating: Some(5),
                    comment: None,
                },
            )
            .await
            .expect("Failed to update review");

        // Verify average is now 5.0
        let updated_avg: Option<f64> = sqlx::query_scalar(
            "SELECT average_rating::float8 FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch updated average");

        assert_eq!(updated_avg, Some(5.0), "Average should be 5.0 for single review");
    }

    // Property 13: Average Rating After Deletion
    // Validates: Requirements 4.4, 5.3
    #[tokio::test]
    async fn test_average_rating_recalculated_after_deletion() {
        let pool = create_test_pool().await;
        let user1 = create_test_user(&pool).await;
        let user2 = create_test_user(&pool).await;
        let user3 = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create three reviews: ratings 5, 4, 3
        // Initial average: (5 + 4 + 3) / 3 = 4.0
        service
            .create_review(
                user1,
                CreateReviewRequest {
                    coffee_id,
                    rating: 5,
                    comment: None,
                },
            )
            .await
            .unwrap();

        let review2 = service
            .create_review(
                user2,
                CreateReviewRequest {
                    coffee_id,
                    rating: 4,
                    comment: None,
                },
            )
            .await
            .unwrap();

        service
            .create_review(
                user3,
                CreateReviewRequest {
                    coffee_id,
                    rating: 3,
                    comment: None,
                },
            )
            .await
            .unwrap();

        // Verify initial average is 4.0
        let initial_avg: Option<f64> = sqlx::query_scalar(
            "SELECT average_rating::float8 FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch initial average");

        assert_eq!(initial_avg, Some(4.0), "Initial average should be 4.0");

        // Delete user2's review (rating 4)
        // New average: (5 + 3) / 2 = 4.0
        service
            .delete_review(review2.id, user2)
            .await
            .expect("Failed to delete review");

        // Verify average was recalculated to 4.0
        let updated_avg: Option<f64> = sqlx::query_scalar(
            "SELECT average_rating::float8 FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch updated average");

        assert_eq!(updated_avg, Some(4.0), "Updated average should be 4.0");

        // Verify review count decreased to 2
        let count: i32 = sqlx::query_scalar(
            "SELECT review_count FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch review count");

        assert_eq!(count, 2, "Review count should be 2 after deletion");
    }

    #[tokio::test]
    async fn test_average_rating_becomes_none_after_deleting_all_reviews() {
        let pool = create_test_pool().await;
        let user1 = create_test_user(&pool).await;
        let user2 = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create two reviews
        let review1 = service
            .create_review(
                user1,
                CreateReviewRequest {
                    coffee_id,
                    rating: 5,
                    comment: None,
                },
            )
            .await
            .unwrap();

        let review2 = service
            .create_review(
                user2,
                CreateReviewRequest {
                    coffee_id,
                    rating: 4,
                    comment: None,
                },
            )
            .await
            .unwrap();

        // Delete first review
        service
            .delete_review(review1.id, user1)
            .await
            .expect("Failed to delete first review");

        // Delete second review
        service
            .delete_review(review2.id, user2)
            .await
            .expect("Failed to delete second review");

        // Verify average is None when no reviews exist
        let avg: Option<f64> = sqlx::query_scalar(
            "SELECT average_rating::float8 FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch average");

        assert_eq!(avg, None, "Average should be None when no reviews exist");

        // Verify review count is 0
        let count: i32 = sqlx::query_scalar(
            "SELECT review_count FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch review count");

        assert_eq!(count, 0, "Review count should be 0");
    }

    #[tokio::test]
    async fn test_average_rating_after_deleting_highest_rating() {
        let pool = create_test_pool().await;
        let user1 = create_test_user(&pool).await;
        let user2 = create_test_user(&pool).await;
        let user3 = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create three reviews: ratings 5, 3, 2
        // Initial average: (5 + 3 + 2) / 3 = 3.333...
        let review1 = service
            .create_review(
                user1,
                CreateReviewRequest {
                    coffee_id,
                    rating: 5,
                    comment: None,
                },
            )
            .await
            .unwrap();

        service
            .create_review(
                user2,
                CreateReviewRequest {
                    coffee_id,
                    rating: 3,
                    comment: None,
                },
            )
            .await
            .unwrap();

        service
            .create_review(
                user3,
                CreateReviewRequest {
                    coffee_id,
                    rating: 2,
                    comment: None,
                },
            )
            .await
            .unwrap();

        // Delete the highest rating (5)
        // New average: (3 + 2) / 2 = 2.5
        service
            .delete_review(review1.id, user1)
            .await
            .expect("Failed to delete review");

        // Verify average decreased to 2.5
        let updated_avg: Option<f64> = sqlx::query_scalar(
            "SELECT average_rating::float8 FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch updated average");

        assert_eq!(updated_avg, Some(2.5), "Average should decrease to 2.5");
    }

    #[tokio::test]
    async fn test_average_rating_after_deleting_lowest_rating() {
        let pool = create_test_pool().await;
        let user1 = create_test_user(&pool).await;
        let user2 = create_test_user(&pool).await;
        let user3 = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create three reviews: ratings 5, 4, 1
        // Initial average: (5 + 4 + 1) / 3 = 3.333...
        service
            .create_review(
                user1,
                CreateReviewRequest {
                    coffee_id,
                    rating: 5,
                    comment: None,
                },
            )
            .await
            .unwrap();

        service
            .create_review(
                user2,
                CreateReviewRequest {
                    coffee_id,
                    rating: 4,
                    comment: None,
                },
            )
            .await
            .unwrap();

        let review3 = service
            .create_review(
                user3,
                CreateReviewRequest {
                    coffee_id,
                    rating: 1,
                    comment: None,
                },
            )
            .await
            .unwrap();

        // Delete the lowest rating (1)
        // New average: (5 + 4) / 2 = 4.5
        service
            .delete_review(review3.id, user3)
            .await
            .expect("Failed to delete review");

        // Verify average increased to 4.5
        let updated_avg: Option<f64> = sqlx::query_scalar(
            "SELECT average_rating::float8 FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch updated average");

        assert_eq!(updated_avg, Some(4.5), "Average should increase to 4.5");
    }

    #[tokio::test]
    async fn test_average_rating_after_deleting_single_review() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create single review with rating 4
        let review = service
            .create_review(
                user_id,
                CreateReviewRequest {
                    coffee_id,
                    rating: 4,
                    comment: None,
                },
            )
            .await
            .unwrap();

        // Delete the only review
        service
            .delete_review(review.id, user_id)
            .await
            .expect("Failed to delete review");

        // Verify average is None
        let avg: Option<f64> = sqlx::query_scalar(
            "SELECT average_rating::float8 FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch average");

        assert_eq!(avg, None, "Average should be None after deleting single review");

        // Verify count is 0
        let count: i32 = sqlx::query_scalar(
            "SELECT review_count FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch review count");

        assert_eq!(count, 0, "Review count should be 0");
    }

    #[tokio::test]
    async fn test_average_rating_with_multiple_deletions() {
        let pool = create_test_pool().await;
        let user1 = create_test_user(&pool).await;
        let user2 = create_test_user(&pool).await;
        let user3 = create_test_user(&pool).await;
        let user4 = create_test_user(&pool).await;
        let coffee_id = create_test_coffee(&pool).await;

        let service = create_service(pool.clone());

        // Create four reviews: ratings 5, 4, 3, 2
        // Initial average: (5 + 4 + 3 + 2) / 4 = 3.5
        let review1 = service
            .create_review(
                user1,
                CreateReviewRequest {
                    coffee_id,
                    rating: 5,
                    comment: None,
                },
            )
            .await
            .unwrap();

        let review2 = service
            .create_review(
                user2,
                CreateReviewRequest {
                    coffee_id,
                    rating: 4,
                    comment: None,
                },
            )
            .await
            .unwrap();

        service
            .create_review(
                user3,
                CreateReviewRequest {
                    coffee_id,
                    rating: 3,
                    comment: None,
                },
            )
            .await
            .unwrap();

        service
            .create_review(
                user4,
                CreateReviewRequest {
                    coffee_id,
                    rating: 2,
                    comment: None,
                },
            )
            .await
            .unwrap();

        // Delete first review (rating 5)
        // New average: (4 + 3 + 2) / 3 = 3.0
        service
            .delete_review(review1.id, user1)
            .await
            .expect("Failed to delete first review");

        let avg_after_first: Option<f64> = sqlx::query_scalar(
            "SELECT average_rating::float8 FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch average");

        assert_eq!(avg_after_first, Some(3.0), "Average should be 3.0 after first deletion");

        // Delete second review (rating 4)
        // New average: (3 + 2) / 2 = 2.5
        service
            .delete_review(review2.id, user2)
            .await
            .expect("Failed to delete second review");

        let avg_after_second: Option<f64> = sqlx::query_scalar(
            "SELECT average_rating::float8 FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch average");

        assert_eq!(avg_after_second, Some(2.5), "Average should be 2.5 after second deletion");

        // Verify count is 2
        let count: i32 = sqlx::query_scalar(
            "SELECT review_count FROM coffees WHERE id = $1"
        )
        .bind(coffee_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch review count");

        assert_eq!(count, 2, "Review count should be 2");
    }
}

