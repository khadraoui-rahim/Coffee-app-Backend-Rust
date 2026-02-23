mod auth;
mod db;
mod models;
mod query;
mod error;
mod validation;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use models::{Coffee, CreateCoffee, UpdateCoffee};
use query::{QueryParams, QueryValidator};
use error::ApiError;
use validator::Validate;

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

/// Handler for POST /api/coffees
/// Creates a new coffee product
#[utoipa::path(
    post,
    path = "/api/coffees",
    request_body = CreateCoffee,
    responses(
        (status = 201, description = "Coffee created successfully", body = Coffee),
        (status = 400, description = "Invalid input data", body = String, example = json!({"error": "Price must be a positive number"})),
        (status = 500, description = "Internal server error", body = String, example = json!({"error": "Database error"}))
    ),
    tag = "coffees"
)]
async fn create_coffee(
    State(state): State<AppState>,
    Json(payload): Json<CreateCoffee>,
) -> Result<(StatusCode, Json<Coffee>), ApiError> {
    tracing::debug!("Creating new coffee: {}", payload.name);
    
    // Validate the request using validator crate
    payload.validate()?;

    // Check for duplicate coffee name
    if db::check_duplicate_coffee(&state.db, &payload.name).await? {
        tracing::warn!("Attempt to create duplicate coffee: {}", payload.name);
        return Err(ApiError::Conflict {
            message: format!("Coffee with name '{}' already exists", payload.name),
        });
    }

    // Insert coffee into database
    let coffee = sqlx::query_as::<_, Coffee>(
        r#"
        INSERT INTO coffees (image_url, name, coffee_type, price, rating)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, image_url, name, coffee_type, price, rating
        "#,
    )
    .bind(&payload.image_url)
    .bind(&payload.name)
    .bind(&payload.coffee_type)
    .bind(payload.price)
    .bind(payload.rating)
    .fetch_one(&state.db)
    .await?;

    tracing::info!("Successfully created coffee with id: {}", coffee.id);
    Ok((StatusCode::CREATED, Json(coffee)))
}

/// Handler for GET /api/coffees
/// Retrieves all coffee products
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
    tracing::debug!("Fetching all coffees");
    
    let coffees = sqlx::query_as::<_, Coffee>(
        r#"
        SELECT id, image_url, name, coffee_type, price, rating
        FROM coffees
        ORDER BY id
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    tracing::debug!("Retrieved {} coffees", coffees.len());
    Ok(Json(coffees))
}

/// Handler for GET /api/coffees with query parameters
/// Supports search, filtering, sorting, and pagination
async fn get_coffees_with_query(
    Query(params): Query<QueryParams>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Coffee>>, ApiError> {
    tracing::debug!("Fetching coffees with query parameters: {:?}", params);
    
    // 1. Validate query parameters
    let validated = QueryValidator::validate(params)
        .map_err(|_e| ApiError::ValidationError(
            validator::ValidationErrors::new()
        ))?;
    
    // 2. Build SQL query
    let mut builder = query::SQLQueryBuilder::new();
    
    // Add filters based on validated params
    if let Some(search) = validated.search {
        builder.add_search_filter(&search);
    }
    if let Some(type_filter) = validated.type_filter {
        builder.add_type_filter(&type_filter);
    }
    builder.add_price_range(validated.min_price, validated.max_price);
    
    // Set sorting if specified
    if let Some(sort_field) = validated.sort_field {
        builder.set_sort(sort_field, validated.sort_order);
    }
    
    // Set pagination
    builder.set_pagination(validated.page, validated.limit);
    
    let (query_str, params) = builder.build();
    
    // 3. Execute query using sqlx with parameterized binding
    let mut query = sqlx::query_as::<_, Coffee>(&query_str);
    
    // Bind all parameters
    for param in params {
        query = query.bind(param);
    }
    
    // Execute query and handle database errors with HTTP 500
    let coffees = query
        .fetch_all(&state.db)
        .await?;
    
    tracing::debug!("Query returned {} coffees", coffees.len());
    
    // Return JSON array of Coffee items with HTTP 200
    Ok(Json(coffees))
}

/// Handler for GET /api/coffees/:id
/// Retrieves a specific coffee product by ID
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
    tracing::debug!("Fetching coffee with id: {}", id);
    
    let coffee = sqlx::query_as::<_, Coffee>(
        r#"
        SELECT id, image_url, name, coffee_type, price, rating
        FROM coffees
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| {
        tracing::debug!("Coffee with id {} not found", id);
        ApiError::NotFound {
            resource: "Coffee".to_string(),
            id: id.to_string(),
        }
    })?;

    tracing::debug!("Successfully retrieved coffee: {}", coffee.name);
    Ok(Json(coffee))
}

/// Handler for PUT /api/coffees/:id
/// Updates an existing coffee product
#[utoipa::path(
    put,
    path = "/api/coffees/{id}",
    params(
        ("id" = i32, Path, description = "Coffee ID")
    ),
    request_body = UpdateCoffee,
    responses(
        (status = 200, description = "Coffee updated successfully", body = Coffee),
        (status = 400, description = "Invalid input data", body = String, example = json!({"error": "Price must be a positive number"})),
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
    tracing::debug!("Updating coffee with id: {}", id);
    
    // Validate the request using validator crate
    payload.validate()?;

    // Use a transaction to ensure atomicity of the multi-step update operation
    // This ensures that if any step fails, all changes are rolled back
    let mut tx = state.db.begin().await?;

    // Check if coffee exists within the transaction
    let existing = sqlx::query_as::<_, Coffee>(
        "SELECT id, image_url, name, coffee_type, price, rating FROM coffees WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| {
        tracing::debug!("Coffee with id {} not found for update", id);
        ApiError::NotFound {
            resource: "Coffee".to_string(),
            id: id.to_string(),
        }
    })?;

    // If name is being updated and it's different from the current name, check for duplicates
    if let Some(ref new_name) = payload.name {
        if new_name != &existing.name {
            // Check for duplicates within the transaction
            let duplicate_exists: Option<bool> = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM coffees WHERE name = $1 AND id != $2)"
            )
            .bind(new_name)
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
            
            if duplicate_exists.unwrap_or(false) {
                tracing::warn!("Attempt to update coffee {} to duplicate name: {}", id, new_name);
                // Transaction will be automatically rolled back when tx is dropped
                return Err(ApiError::Conflict {
                    message: format!("Coffee with name '{}' already exists", new_name),
                });
            }
        }
    }

    // Update coffee with provided fields, keeping existing values for omitted fields
    let updated_coffee = sqlx::query_as::<_, Coffee>(
        r#"
        UPDATE coffees
        SET image_url = $1,
            name = $2,
            coffee_type = $3,
            price = $4,
            rating = $5
        WHERE id = $6
        RETURNING id, image_url, name, coffee_type, price, rating
        "#,
    )
    .bind(payload.image_url.unwrap_or(existing.image_url))
    .bind(payload.name.unwrap_or(existing.name))
    .bind(payload.coffee_type.unwrap_or(existing.coffee_type))
    .bind(payload.price.unwrap_or(existing.price))
    .bind(payload.rating.unwrap_or(existing.rating))
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    // Commit the transaction - if this fails, changes are rolled back
    tx.commit().await?;

    tracing::info!("Successfully updated coffee with id: {}", id);
    Ok(Json(updated_coffee))
}

/// Handler for DELETE /api/coffees/:id
/// Deletes a coffee product
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
    tracing::debug!("Deleting coffee with id: {}", id);
    
    let result = sqlx::query("DELETE FROM coffees WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        tracing::debug!("Coffee with id {} not found for deletion", id);
        return Err(ApiError::NotFound {
            resource: "Coffee".to_string(),
            id: id.to_string(),
        });
    }

    tracing::info!("Successfully deleted coffee with id: {}", id);
    Ok(StatusCode::NO_CONTENT)
}

/// Creates and configures the application router
/// Maps all API endpoints to their handlers and adds CORS middleware
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
        .route("/api/coffees", get(get_coffees_with_query))
        .route("/api/coffees/:id", get(get_coffee_by_id))
        .route("/api/coffees/:id", put(update_coffee))
        .route("/api/coffees/:id", delete(delete_coffee))
        .layer(cors)
        .with_state(state)
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Initialize tracing subscriber for logging
    // This enables the error!, warn!, info!, debug!, and trace! macros
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    tracing::info!("Coffee API - Starting...");

    // Get configuration from environment variables
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in environment");
    let host = std::env::var("HOST")
        .unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string());

    // Create database connection pool
    tracing::info!("Connecting to database...");
    let db_pool = db::create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    // Run SQLx migrations on startup
    tracing::info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run database migrations");
    tracing::info!("Migrations completed successfully");

    // Create the application router
    let app = create_router(db_pool);

    // Start the Axum server
    let addr = format!("{}:{}", host, port);
    tracing::info!("Starting server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    tracing::info!("Coffee API is running on http://{}", addr);
    tracing::info!("Swagger UI available at http://{}/swagger-ui", addr);
    
    axum::serve(listener, app)
        .await
        .expect("Server error");
}

#[cfg(test)]
mod tests;
