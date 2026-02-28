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
    let email = format!("test{}@example.com", timestamp);
    
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
    let name = format!("Test Coffee {}", timestamp);
    
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

// ============================================================================
// Repository CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_review_success() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let coffee_id = create_test_coffee(&pool).await;
    
    let repository = ReviewRepository::new(pool.clone());
    
    let review = repository
        .create(user_id, coffee_id, 5, Some("Great coffee!".to_string()))
        .await
        .expect("Failed to create review");
    
    assert!(review.id > 0);
    assert_eq!(review.user_id, user_id);
    assert_eq!(review.coffee_id, coffee_id);
    assert_eq!(review.rating, 5);
    assert_eq!(review.comment, Some("Great coffee!".to_string()));
    assert!(review.created_at <= chrono::Utc::now());
    assert!(review.updated_at <= chrono::Utc::now());
}

#[tokio::test]
async fn test_find_by_id_existing() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let coffee_id = create_test_coffee(&pool).await;
    
    let repository = ReviewRepository::new(pool.clone());
    
    let created = repository
        .create(user_id, coffee_id, 4, Some("Good".to_string()))
        .await
        .expect("Failed to create review");
    
    let found = repository
        .find_by_id(created.id)
        .await
        .expect("Failed to find review")
        .expect("Review not found");
    
    assert_eq!(found.id, created.id);
    assert_eq!(found.user_id, user_id);
    assert_eq!(found.coffee_id, coffee_id);
    assert_eq!(found.rating, 4);
}

#[tokio::test]
async fn test_find_by_id_non_existent() {
    let pool = create_test_pool().await;
    let repository = ReviewRepository::new(pool.clone());
    
    let result = repository
        .find_by_id(99999)
        .await
        .expect("Query failed");
    
    assert!(result.is_none());
}

#[tokio::test]
async fn test_find_by_user_and_coffee_existing() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let coffee_id = create_test_coffee(&pool).await;
    
    let repository = ReviewRepository::new(pool.clone());
    
    repository
        .create(user_id, coffee_id, 5, None)
        .await
        .expect("Failed to create review");
    
    let found = repository
        .find_by_user_and_coffee(user_id, coffee_id)
        .await
        .expect("Failed to find review")
        .expect("Review not found");
    
    assert_eq!(found.user_id, user_id);
    assert_eq!(found.coffee_id, coffee_id);
}

#[tokio::test]
async fn test_find_by_user_and_coffee_non_existent() {
    let pool = create_test_pool().await;
    let repository = ReviewRepository::new(pool.clone());
    
    let result = repository
        .find_by_user_and_coffee(99999, 99999)
        .await
        .expect("Query failed");
    
    assert!(result.is_none());
}

#[tokio::test]
async fn test_update_review_rating_and_comment() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let coffee_id = create_test_coffee(&pool).await;
    
    let repository = ReviewRepository::new(pool.clone());
    
    let created = repository
        .create(user_id, coffee_id, 3, Some("OK".to_string()))
        .await
        .expect("Failed to create review");
    
    let updated = repository
        .update(created.id, Some(5), Some("Excellent!".to_string()))
        .await
        .expect("Failed to update review");
    
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.rating, 5);
    assert_eq!(updated.comment, Some("Excellent!".to_string()));
    assert!(updated.updated_at > created.updated_at);
}

#[tokio::test]
async fn test_update_review_comment_only() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let coffee_id = create_test_coffee(&pool).await;
    
    let repository = ReviewRepository::new(pool.clone());
    
    let created = repository
        .create(user_id, coffee_id, 4, Some("Good".to_string()))
        .await
        .expect("Failed to create review");
    
    let updated = repository
        .update(created.id, None, Some("Very good!".to_string()))
        .await
        .expect("Failed to update review");
    
    assert_eq!(updated.rating, 4); // Rating unchanged
    assert_eq!(updated.comment, Some("Very good!".to_string()));
}

#[tokio::test]
async fn test_delete_review_success() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let coffee_id = create_test_coffee(&pool).await;
    
    let repository = ReviewRepository::new(pool.clone());
    
    let created = repository
        .create(user_id, coffee_id, 5, None)
        .await
        .expect("Failed to create review");
    
    repository
        .delete(created.id)
        .await
        .expect("Failed to delete review");
    
    let result = repository
        .find_by_id(created.id)
        .await
        .expect("Query failed");
    
    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_review_non_existent() {
    let pool = create_test_pool().await;
    let repository = ReviewRepository::new(pool.clone());
    
    let result = repository.delete(99999).await;
    
    assert!(result.is_err());
    match result {
        Err(ServiceError::NotFound) => (),
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_find_by_coffee() {
    let pool = create_test_pool().await;
    let user1 = create_test_user(&pool).await;
    let user2 = create_test_user(&pool).await;
    let coffee_id = create_test_coffee(&pool).await;
    
    let repository = ReviewRepository::new(pool.clone());
    
    repository.create(user1, coffee_id, 5, Some("Great!".to_string())).await.unwrap();
    repository.create(user2, coffee_id, 4, Some("Good".to_string())).await.unwrap();
    
    let reviews = repository
        .find_by_coffee(coffee_id)
        .await
        .expect("Failed to find reviews");
    
    assert_eq!(reviews.len(), 2);
}

#[tokio::test]
async fn test_find_by_coffee_no_reviews() {
    let pool = create_test_pool().await;
    let coffee_id = create_test_coffee(&pool).await;
    let repository = ReviewRepository::new(pool.clone());
    
    let reviews = repository
        .find_by_coffee(coffee_id)
        .await
        .expect("Failed to find reviews");
    
    assert_eq!(reviews.len(), 0);
}

#[tokio::test]
async fn test_get_ratings_for_coffee() {
    let pool = create_test_pool().await;
    let user1 = create_test_user(&pool).await;
    let user2 = create_test_user(&pool).await;
    let user3 = create_test_user(&pool).await;
    let coffee_id = create_test_coffee(&pool).await;
    
    let repository = ReviewRepository::new(pool.clone());
    
    repository.create(user1, coffee_id, 5, None).await.unwrap();
    repository.create(user2, coffee_id, 4, None).await.unwrap();
    repository.create(user3, coffee_id, 3, None).await.unwrap();
    
    let ratings = repository
        .get_ratings_for_coffee(coffee_id)
        .await
        .expect("Failed to get ratings");
    
    assert_eq!(ratings.len(), 3);
    assert!(ratings.contains(&5));
    assert!(ratings.contains(&4));
    assert!(ratings.contains(&3));
}

#[tokio::test]
async fn test_update_coffee_rating() {
    let pool = create_test_pool().await;
    let coffee_id = create_test_coffee(&pool).await;
    let repository = ReviewRepository::new(pool.clone());
    
    repository
        .update_coffee_rating(coffee_id, Some(4.5), 10)
        .await
        .expect("Failed to update coffee rating");
    
    // Query the raw values to verify they were set
    let count: i32 = sqlx::query_scalar(
        "SELECT review_count FROM coffees WHERE id = $1"
    )
    .bind(coffee_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch review count");
    
    assert_eq!(count, 10);
    
    // Verify average_rating is not null
    let has_rating: bool = sqlx::query_scalar(
        "SELECT average_rating IS NOT NULL FROM coffees WHERE id = $1"
    )
    .bind(coffee_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to check rating");
    
    assert!(has_rating);
}

#[tokio::test]
async fn test_coffee_exists_true() {
    let pool = create_test_pool().await;
    let coffee_id = create_test_coffee(&pool).await;
    let repository = ReviewRepository::new(pool.clone());
    
    let exists = repository
        .coffee_exists(coffee_id)
        .await
        .expect("Failed to check coffee existence");
    
    assert!(exists);
}

#[tokio::test]
async fn test_coffee_exists_false() {
    let pool = create_test_pool().await;
    let repository = ReviewRepository::new(pool.clone());
    
    let exists = repository
        .coffee_exists(99999)
        .await
        .expect("Failed to check coffee existence");
    
    assert!(!exists);
}
