use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// Domain model representing a review in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Review {
    pub id: i32,
    pub user_id: i32,
    pub coffee_id: i32,
    pub rating: i16,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request DTO for creating a new review
#[derive(Debug, Deserialize, Validate, Clone)]
pub struct CreateReviewRequest {
    pub coffee_id: i32,
    #[validate(range(min = 1, max = 5, message = "Rating must be between 1 and 5"))]
    pub rating: i16,
    #[validate(length(max = 1000, message = "Comment must not exceed 1000 characters"))]
    pub comment: Option<String>,
}

/// Request DTO for updating an existing review
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateReviewRequest {
    #[validate(range(min = 1, max = 5, message = "Rating must be between 1 and 5"))]
    pub rating: Option<i16>,
    #[validate(length(max = 1000, message = "Comment must not exceed 1000 characters"))]
    pub comment: Option<String>,
}

/// Response DTO for API responses
#[derive(Debug, Serialize)]
pub struct ReviewResponse {
    pub id: i32,
    pub user_id: i32,
    pub coffee_id: i32,
    pub rating: i16,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Review> for ReviewResponse {
    fn from(review: Review) -> Self {
        Self {
            id: review.id,
            user_id: review.user_id,
            coffee_id: review.coffee_id,
            rating: review.rating,
            comment: review.comment,
            created_at: review.created_at,
            updated_at: review.updated_at,
        }
    }
}
