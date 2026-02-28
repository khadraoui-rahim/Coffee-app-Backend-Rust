use sqlx::PgPool;
use crate::reviews::{Review, ServiceError};

/// Repository for database operations on reviews
#[derive(Clone)]
pub struct ReviewRepository {
    pool: PgPool,
}

impl ReviewRepository {
    /// Create a new ReviewRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new review
    pub async fn create(
        &self,
        user_id: i32,
        coffee_id: i32,
        rating: i16,
        comment: Option<String>,
    ) -> Result<Review, ServiceError> {
        let review = sqlx::query_as::<_, Review>(
            r#"
            INSERT INTO reviews (user_id, coffee_id, rating, comment)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, coffee_id, rating, comment, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(coffee_id)
        .bind(rating)
        .bind(comment)
        .fetch_one(&self.pool)
        .await?;

        Ok(review)
    }

    /// Find a review by ID
    pub async fn find_by_id(&self, id: i32) -> Result<Option<Review>, ServiceError> {
        let review = sqlx::query_as::<_, Review>(
            r#"
            SELECT id, user_id, coffee_id, rating, comment, created_at, updated_at
            FROM reviews
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(review)
    }

    /// Find a review by user_id and coffee_id (for duplicate detection)
    pub async fn find_by_user_and_coffee(
        &self,
        user_id: i32,
        coffee_id: i32,
    ) -> Result<Option<Review>, ServiceError> {
        let review = sqlx::query_as::<_, Review>(
            r#"
            SELECT id, user_id, coffee_id, rating, comment, created_at, updated_at
            FROM reviews
            WHERE user_id = $1 AND coffee_id = $2
            "#,
        )
        .bind(user_id)
        .bind(coffee_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(review)
    }

    /// Update a review
    pub async fn update(
        &self,
        id: i32,
        rating: Option<i16>,
        comment: Option<String>,
    ) -> Result<Review, ServiceError> {
        // Build dynamic update query based on what fields are provided
        let review = match (rating, comment) {
            (Some(new_rating), Some(new_comment)) => {
                // Update both rating and comment
                sqlx::query_as::<_, Review>(
                    r#"
                    UPDATE reviews
                    SET rating = $1, comment = $2, updated_at = NOW()
                    WHERE id = $3
                    RETURNING id, user_id, coffee_id, rating, comment, created_at, updated_at
                    "#,
                )
                .bind(new_rating)
                .bind(new_comment)
                .bind(id)
                .fetch_one(&self.pool)
                .await?
            }
            (Some(new_rating), None) => {
                // Update only rating
                sqlx::query_as::<_, Review>(
                    r#"
                    UPDATE reviews
                    SET rating = $1, updated_at = NOW()
                    WHERE id = $2
                    RETURNING id, user_id, coffee_id, rating, comment, created_at, updated_at
                    "#,
                )
                .bind(new_rating)
                .bind(id)
                .fetch_one(&self.pool)
                .await?
            }
            (None, Some(new_comment)) => {
                // Update only comment
                sqlx::query_as::<_, Review>(
                    r#"
                    UPDATE reviews
                    SET comment = $1, updated_at = NOW()
                    WHERE id = $2
                    RETURNING id, user_id, coffee_id, rating, comment, created_at, updated_at
                    "#,
                )
                .bind(new_comment)
                .bind(id)
                .fetch_one(&self.pool)
                .await?
            }
            (None, None) => {
                // No fields to update, just return the existing review
                // But still update the timestamp
                sqlx::query_as::<_, Review>(
                    r#"
                    UPDATE reviews
                    SET updated_at = NOW()
                    WHERE id = $1
                    RETURNING id, user_id, coffee_id, rating, comment, created_at, updated_at
                    "#,
                )
                .bind(id)
                .fetch_one(&self.pool)
                .await?
            }
        };

        Ok(review)
    }

    /// Delete a review
    pub async fn delete(&self, id: i32) -> Result<(), ServiceError> {
        let result = sqlx::query("DELETE FROM reviews WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(ServiceError::NotFound);
        }

        Ok(())
    }

    /// Find all reviews for a coffee
    pub async fn find_by_coffee(&self, coffee_id: i32) -> Result<Vec<Review>, ServiceError> {
        let reviews = sqlx::query_as::<_, Review>(
            r#"
            SELECT id, user_id, coffee_id, rating, comment, created_at, updated_at
            FROM reviews
            WHERE coffee_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(coffee_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(reviews)
    }

    /// Get all rating values for a coffee (for average calculation)
    pub async fn get_ratings_for_coffee(&self, coffee_id: i32) -> Result<Vec<i16>, ServiceError> {
        let ratings: Vec<(i16,)> = sqlx::query_as(
            r#"
            SELECT rating
            FROM reviews
            WHERE coffee_id = $1
            "#,
        )
        .bind(coffee_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(ratings.into_iter().map(|(r,)| r).collect())
    }

    /// Update the average rating and review count for a coffee
    pub async fn update_coffee_rating(
        &self,
        coffee_id: i32,
        average: Option<f64>,
        count: i32,
    ) -> Result<(), ServiceError> {
        sqlx::query(
            r#"
            UPDATE coffees
            SET average_rating = $1, review_count = $2
            WHERE id = $3
            "#,
        )
        .bind(average)
        .bind(count)
        .bind(coffee_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Check if a coffee exists
    pub async fn coffee_exists(&self, coffee_id: i32) -> Result<bool, ServiceError> {
        let exists: Option<bool> = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM coffees WHERE id = $1)"
        )
        .bind(coffee_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists.unwrap_or(false))
    }
}
