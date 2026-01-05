mod db;
mod models;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post, put},
    Router,
};
use serde_json::json;
use sqlx::PgPool;

use models::{Coffee, CreateCoffee, UpdateCoffee};

/// Application state shared across handlers
#[derive(Clone)]
struct AppState {
    db: PgPool,
}

/// Custom error type for API responses
/// Implements IntoResponse to convert errors into HTTP responses
#[derive(Debug)]
enum ApiError {
    NotFound(String),
    BadRequest(String),
    InternalError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

/// Handler for POST /api/coffees
/// Creates a new coffee product
/// 
/// Requirements: 2.1, 2.2, 2.3, 2.4
async fn create_coffee(
    State(state): State<AppState>,
    Json(payload): Json<CreateCoffee>,
) -> Result<(StatusCode, Json<Coffee>), ApiError> {
    // Validate price (Requirement 8.4)
    if payload.price <= 0 {
        return Err(ApiError::BadRequest(
            "Price must be a positive integer".to_string(),
        ));
    }

    // Validate rating (Requirement 8.5)
    if payload.rating < 0.0 || payload.rating > 5.0 {
        return Err(ApiError::BadRequest(
            "Rating must be between 0.0 and 5.0".to_string(),
        ));
    }

    // Validate temperature (Requirement 8.6)
    if !["hot", "cold", "both"].contains(&payload.temperature.as_str()) {
        return Err(ApiError::BadRequest(
            "Temperature must be one of: hot, cold, both".to_string(),
        ));
    }

    // Insert coffee into database
    let coffee = sqlx::query_as::<_, Coffee>(
        r#"
        INSERT INTO coffees (name, coffee_type, price, rating, temperature, description, size, liked)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, name, coffee_type, price, rating, temperature, description, size, liked, created_at, updated_at
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.coffee_type)
    .bind(payload.price)
    .bind(payload.rating)
    .bind(&payload.temperature)
    .bind(&payload.description)
    .bind(&payload.size)
    .bind(payload.liked)
    .fetch_one(&state.db)
    .await
    .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

    Ok((StatusCode::CREATED, Json(coffee)))
}

/// Handler for GET /api/coffees
/// Retrieves all coffee products
/// 
/// Requirements: 3.1, 3.4
async fn get_all_coffees(
    State(state): State<AppState>,
) -> Result<Json<Vec<Coffee>>, ApiError> {
    let coffees = sqlx::query_as::<_, Coffee>(
        r#"
        SELECT id, name, coffee_type, price, rating, temperature, description, size, liked, created_at, updated_at
        FROM coffees
        ORDER BY id
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

    Ok(Json(coffees))
}

/// Handler for GET /api/coffees/:id
/// Retrieves a specific coffee product by ID
/// 
/// Requirements: 3.2, 3.3, 3.4
async fn get_coffee_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Coffee>, ApiError> {
    let coffee = sqlx::query_as::<_, Coffee>(
        r#"
        SELECT id, name, coffee_type, price, rating, temperature, description, size, liked, created_at, updated_at
        FROM coffees
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

    match coffee {
        Some(coffee) => Ok(Json(coffee)),
        None => Err(ApiError::NotFound(format!("Coffee with id {} not found", id))),
    }
}

/// Handler for PUT /api/coffees/:id
/// Updates an existing coffee product
/// 
/// Requirements: 4.1, 4.2, 4.3, 4.4, 4.5
async fn update_coffee(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateCoffee>,
) -> Result<Json<Coffee>, ApiError> {
    // Validate price if provided (Requirement 8.4)
    if let Some(price) = payload.price {
        if price <= 0 {
            return Err(ApiError::BadRequest(
                "Price must be a positive integer".to_string(),
            ));
        }
    }

    // Validate rating if provided (Requirement 8.5)
    if let Some(rating) = payload.rating {
        if rating < 0.0 || rating > 5.0 {
            return Err(ApiError::BadRequest(
                "Rating must be between 0.0 and 5.0".to_string(),
            ));
        }
    }

    // Validate temperature if provided (Requirement 8.6)
    if let Some(ref temperature) = payload.temperature {
        if !["hot", "cold", "both"].contains(&temperature.as_str()) {
            return Err(ApiError::BadRequest(
                "Temperature must be one of: hot, cold, both".to_string(),
            ));
        }
    }

    // Check if coffee exists
    let existing = sqlx::query_as::<_, Coffee>(
        "SELECT id, name, coffee_type, price, rating, temperature, description, size, liked, created_at, updated_at FROM coffees WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

    let existing = match existing {
        Some(coffee) => coffee,
        None => return Err(ApiError::NotFound(format!("Coffee with id {} not found", id))),
    };

    // Update coffee with provided fields, keeping existing values for omitted fields
    let updated_coffee = sqlx::query_as::<_, Coffee>(
        r#"
        UPDATE coffees
        SET name = $1,
            coffee_type = $2,
            price = $3,
            rating = $4,
            temperature = $5,
            description = $6,
            size = $7,
            liked = $8,
            updated_at = NOW()
        WHERE id = $9
        RETURNING id, name, coffee_type, price, rating, temperature, description, size, liked, created_at, updated_at
        "#,
    )
    .bind(payload.name.unwrap_or(existing.name))
    .bind(payload.coffee_type.unwrap_or(existing.coffee_type))
    .bind(payload.price.unwrap_or(existing.price))
    .bind(payload.rating.unwrap_or(existing.rating))
    .bind(payload.temperature.unwrap_or(existing.temperature))
    .bind(payload.description.unwrap_or(existing.description))
    .bind(payload.size.unwrap_or(existing.size))
    .bind(payload.liked.unwrap_or(existing.liked))
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

    Ok(Json(updated_coffee))
}

/// Handler for DELETE /api/coffees/:id
/// Deletes a coffee product
/// 
/// Requirements: 5.1, 5.2, 5.3
async fn delete_coffee(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query("DELETE FROM coffees WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound(format!("Coffee with id {} not found", id)));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[tokio::main]
async fn main() {
    println!("Coffee API - Starting...");
    
    // This is a placeholder main function
    // The actual server setup will be implemented in task 6
    println!("CRUD handlers implemented!");
    println!("Ready for routing setup in task 6");
}

#[cfg(test)]
mod tests;
