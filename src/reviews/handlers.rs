// HTTP handlers for review endpoints

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use validator::Validate;

use crate::auth::middleware::AuthenticatedUser;
use crate::reviews::{
    error::ErrorResponse,
    models::{CreateReviewRequest, ReviewResponse, UpdateReviewRequest},
    ServiceError,
};
use crate::AppState;

/// Create a new review
/// POST /api/reviews
pub async fn create_review_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<CreateReviewRequest>,
) -> Result<(StatusCode, Json<ReviewResponse>), ErrorResponse> {
    // Validate request
    request
        .validate()
        .map_err(|e| ServiceError::ValidationError(e.to_string()))?;

    // Create review
    let review = state.review_service.create_review(user.user_id, request).await?;

    // Convert to response
    let response = ReviewResponse {
        id: review.id,
        user_id: review.user_id,
        coffee_id: review.coffee_id,
        rating: review.rating,
        comment: review.comment,
        created_at: review.created_at,
        updated_at: review.updated_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Update an existing review
/// PUT /api/reviews/{id}
pub async fn update_review_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(review_id): Path<i32>,
    Json(request): Json<UpdateReviewRequest>,
) -> Result<Json<ReviewResponse>, ErrorResponse> {
    // Validate request
    request
        .validate()
        .map_err(|e| ServiceError::ValidationError(e.to_string()))?;

    // Update review
    let review = state.review_service
        .update_review(review_id, user.user_id, request)
        .await?;

    // Convert to response
    let response = ReviewResponse {
        id: review.id,
        user_id: review.user_id,
        coffee_id: review.coffee_id,
        rating: review.rating,
        comment: review.comment,
        created_at: review.created_at,
        updated_at: review.updated_at,
    };

    Ok(Json(response))
}

/// Delete a review
/// DELETE /api/reviews/{id}
pub async fn delete_review_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(review_id): Path<i32>,
) -> Result<StatusCode, ErrorResponse> {
    // Delete review
    state.review_service.delete_review(review_id, user.user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get all reviews for a coffee
/// GET /api/coffees/{id}/reviews
pub async fn get_reviews_for_coffee_handler(
    State(state): State<AppState>,
    Path(coffee_id): Path<i32>,
) -> Result<Json<Vec<ReviewResponse>>, ErrorResponse> {
    // Get reviews
    let reviews = state.review_service.get_reviews_for_coffee(coffee_id).await?;

    // Convert to responses
    let responses: Vec<ReviewResponse> = reviews
        .into_iter()
        .map(|review| ReviewResponse {
            id: review.id,
            user_id: review.user_id,
            coffee_id: review.coffee_id,
            rating: review.rating,
            comment: review.comment,
            created_at: review.created_at,
            updated_at: review.updated_at,
        })
        .collect();

    Ok(Json(responses))
}
