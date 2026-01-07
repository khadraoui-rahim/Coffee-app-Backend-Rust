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
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use models::{Coffee, CreateCoffee, UpdateCoffee};

/// OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    paths(
        create_coffee,
        get_all_coffees,
        get_coffee_by_id,
        update_coffee,
        delete_coffee,
    ),
    components(
        schemas(Coffee, CreateCoffee, UpdateCoffee)
    ),
    tags(
        (name = "coffees", description = "Coffee menu management endpoints")
    ),
    info(
        title = "Coffee Menu API",
        version = "1.0.0",
        description = "RESTful API for managing coffee menu items",
        contact(
            name = "API Support",
            email = "support@coffeeapi.com"
        )
    )
)]
struct ApiDoc;

/// Application state shared across handlers
#[derive(Clone)]
struct AppState {
    db: PgPool,
}

/// Validation functions
/// Requirements: 8.4, 8.5, 8.6

/// Validates that price is a positive integer
/// Requirement 8.4: Price must be > 0
fn validate_price(price: i32) -> Result<(), String> {
    if price <= 0 {
        Err("Price must be a positive integer".to_string())
    } else {
        Ok(())
    }
}

/// Validates that rating is between 0.0 and 5.0
/// Requirement 8.5: Rating must be between 0.0 and 5.0
fn validate_rating(rating: f64) -> Result<(), String> {
    if rating < 0.0 || rating > 5.0 {
        Err("Rating must be between 0.0 and 5.0".to_string())
    } else {
        Ok(())
    }
}

/// Validates that temperature is one of "hot", "cold", or "both"
/// Requirement 8.6: Temperature must be one of "hot", "cold", or "both"
fn validate_temperature(temperature: &str) -> Result<(), String> {
    if !["hot", "cold", "both"].contains(&temperature) {
        Err("Temperature must be one of: hot, cold, both".to_string())
    } else {
        Ok(())
    }
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
#[utoipa::path(
    post,
    path = "/api/coffees",
    request_body = CreateCoffee,
    responses(
        (status = 201, description = "Coffee created successfully", body = Coffee),
        (status = 400, description = "Invalid input data", body = String, example = json!({"error": "Price must be a positive integer"})),
        (status = 500, description = "Internal server error", body = String, example = json!({"error": "Database error"}))
    ),
    tag = "coffees"
)]
async fn create_coffee(
    State(state): State<AppState>,
    Json(payload): Json<CreateCoffee>,
) -> Result<(StatusCode, Json<Coffee>), ApiError> {
    // Validate price (Requirement 8.4)
    validate_price(payload.price)
        .map_err(|e| ApiError::BadRequest(e))?;

    // Validate rating (Requirement 8.5)
    validate_rating(payload.rating)
        .map_err(|e| ApiError::BadRequest(e))?;

    // Validate temperature (Requirement 8.6)
    validate_temperature(&payload.temperature)
        .map_err(|e| ApiError::BadRequest(e))?;

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
#[utoipa::path(
    get,
    path = "/api/coffees",
    responses(
        (status = 200, description = "List of all coffees", body = Vec<Coffee>),
        (status = 500, description = "Internal server error", body = String, example = json!({"error": "Database error"}))
    ),
    tag = "coffees"
)]
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
#[utoipa::path(
    get,
    path = "/api/coffees/{id}",
    params(
        ("id" = i32, Path, description = "Coffee ID")
    ),
    responses(
        (status = 200, description = "Coffee found", body = Coffee),
        (status = 404, description = "Coffee not found", body = String, example = json!({"error": "Coffee with id 1 not found"})),
        (status = 500, description = "Internal server error", body = String, example = json!({"error": "Database error"}))
    ),
    tag = "coffees"
)]
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
#[utoipa::path(
    put,
    path = "/api/coffees/{id}",
    params(
        ("id" = i32, Path, description = "Coffee ID")
    ),
    request_body = UpdateCoffee,
    responses(
        (status = 200, description = "Coffee updated successfully", body = Coffee),
        (status = 400, description = "Invalid input data", body = String, example = json!({"error": "Price must be a positive integer"})),
        (status = 404, description = "Coffee not found", body = String, example = json!({"error": "Coffee with id 1 not found"})),
        (status = 500, description = "Internal server error", body = String, example = json!({"error": "Database error"}))
    ),
    tag = "coffees"
)]
async fn update_coffee(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateCoffee>,
) -> Result<Json<Coffee>, ApiError> {
    // Validate price if provided (Requirement 8.4)
    if let Some(price) = payload.price {
        validate_price(price)
            .map_err(|e| ApiError::BadRequest(e))?;
    }

    // Validate rating if provided (Requirement 8.5)
    if let Some(rating) = payload.rating {
        validate_rating(rating)
            .map_err(|e| ApiError::BadRequest(e))?;
    }

    // Validate temperature if provided (Requirement 8.6)
    if let Some(ref temperature) = payload.temperature {
        validate_temperature(temperature)
            .map_err(|e| ApiError::BadRequest(e))?;
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
#[utoipa::path(
    delete,
    path = "/api/coffees/{id}",
    params(
        ("id" = i32, Path, description = "Coffee ID")
    ),
    responses(
        (status = 204, description = "Coffee deleted successfully"),
        (status = 404, description = "Coffee not found", body = String, example = json!({"error": "Coffee with id 1 not found"})),
        (status = 500, description = "Internal server error", body = String, example = json!({"error": "Database error"}))
    ),
    tag = "coffees"
)]
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

/// Creates and configures the application router
/// Maps all API endpoints to their handlers and adds CORS middleware
/// 
/// Requirements: 2.1-2.4, 3.1-3.4, 4.1-4.5, 5.1-5.3
fn create_router(db: PgPool) -> Router {
    use tower_http::cors::{CorsLayer, Any};

    let state = AppState { db };

    // Configure CORS to allow all origins, methods, and headers
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", ApiDoc::openapi()))
        // API routes
        .route("/api/coffees", post(create_coffee))
        .route("/api/coffees", get(get_all_coffees))
        .route("/api/coffees/:id", get(get_coffee_by_id))
        .route("/api/coffees/:id", put(update_coffee))
        .route("/api/coffees/:id", delete(delete_coffee))
        .layer(cors)
        .with_state(state)
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    // Requirement 7.4: Load configuration from environment
    dotenv::dotenv().ok();

    println!("Coffee API - Starting...");

    // Get configuration from environment variables
    // Requirement 7.4: Use environment variables for configuration
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in environment");
    let host = std::env::var("HOST")
        .unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string());

    // Create database connection pool
    // Requirement 6.1: Create database connection pool
    println!("Connecting to database...");
    let db_pool = db::create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    // Run SQLx migrations on startup
    // Requirement 6.5: Automatically run pending migrations
    println!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run database migrations");
    println!("Migrations completed successfully");

    // Create the application router
    let app = create_router(db_pool);

    // Start the Axum server
    // Requirements: 7.3, 7.5: Start server on configured host and port
    let addr = format!("{}:{}", host, port);
    println!("Starting server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    println!("Coffee API is running on http://{}", addr);
    println!("Swagger UI available at http://{}/swagger-ui", addr);
    
    axum::serve(listener, app)
        .await
        .expect("Server error");
}

#[cfg(test)]
mod tests;
