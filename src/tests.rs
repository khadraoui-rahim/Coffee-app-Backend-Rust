// Handler tests for Coffee Menu Backend API
// This module contains comprehensive unit tests for all CRUD operations

use super::*;
use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::json;
use sqlx::PgPool;

// ============================================================================
// Test Helpers
// ============================================================================

/// Helper function to create a test database pool
/// Connects to the TEST database, runs migrations, and cleans test data
/// Uses transactions to ensure test isolation
async fn create_test_pool() -> PgPool {
    // Use TEST_DATABASE_URL if available, otherwise fall back to a test database URL
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://coffee_user:coffee_pass@test_db:5432/coffee_test_db".to_string());
    
    println!("Connecting to test database: {}", database_url);
    
    let pool = crate::db::create_pool(&database_url)
        .await
        .expect("Failed to connect to test database");
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    pool
}

/// Helper function to clean test data before each test
/// This should be called at the start of each test to ensure isolation
async fn clean_test_data(pool: &PgPool) {
    sqlx::query("TRUNCATE TABLE coffees RESTART IDENTITY CASCADE")
        .execute(pool)
        .await
        .expect("Failed to clean test data");
}

/// Helper function to create a test app with database
async fn create_test_app(pool: PgPool) -> TestServer {
    // Set JWT_SECRET for middleware (required by RequireRole middleware)
    std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");
    
    // Clean test data to ensure isolation
    clean_test_data(&pool).await;
    
    // Initialize auth service for tests
    let jwt_secret = "test_secret_key_for_testing_purposes".to_string();
    let token_service = crate::auth::token::TokenService::new(jwt_secret);
    let password_service = crate::auth::password::PasswordService;
    let user_repository = crate::auth::repository::UserRepository::new(pool.clone());
    let token_repository = crate::auth::repository::TokenRepository::new(pool.clone());
    let auth_service = std::sync::Arc::new(crate::auth::service::AuthService::new(
        user_repository,
        token_repository,
        password_service,
        token_service,
    ));
    
    // Initialize review service
    let review_repository = crate::reviews::ReviewRepository::new(pool.clone());
    let rating_calculator = crate::reviews::RatingCalculator::new(review_repository.clone());
    let review_service = crate::reviews::ReviewService::new(review_repository, rating_calculator);
    
    let state = AppState { 
        db: pool.clone(),
        auth_service: auth_service.clone(),
        review_service,
    };
    
    use axum::middleware::from_fn;
    
    // Create protected admin routes with RequireRole middleware
    let admin_routes = Router::new()
        .route("/api/coffees", post(create_coffee))
        .route("/api/coffees/:id", put(update_coffee))
        .route("/api/coffees/:id", delete(delete_coffee))
        .route_layer(from_fn(move |req, next| {
            crate::auth::middleware::RequireRole::admin().middleware(req, next)
        }));
    
    // Create public routes
    let public_routes = Router::new()
        .route("/api/coffees", get(get_all_coffees))
        .route("/api/coffees/:id", get(get_coffee_by_id));
    
    let app = Router::new()
        .merge(admin_routes)
        .merge(public_routes)
        .route("/api/auth/register", post(crate::auth::handlers::register_handler))
        .route("/api/auth/login", post(crate::auth::handlers::login_handler))
        .route("/api/auth/refresh", post(crate::auth::handlers::refresh_handler))
        .route("/api/auth/me", get(crate::auth::handlers::me_handler))
        .with_state(state);

    TestServer::new(app).unwrap()
}

/// Helper function to create a valid coffee payload for testing
fn create_valid_coffee_payload(name: &str) -> serde_json::Value {
    json!({
        "image_url": "https://images.unsplash.com/photo-test",
        "name": name,
        "coffee_type": "Test Type",
        "price": 3.50,
        "rating": 4.5
    })
}

// ============================================================================
// CREATE Coffee Tests (POST /api/coffees)
// ============================================================================

/// Test successful coffee creation with all valid fields
#[tokio::test]
async fn test_create_coffee_success() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    let payload = json!({
        "image_url": "https://images.unsplash.com/photo-1594146971821-373461fd5cd8",
        "name": "Espresso",
        "coffee_type": "Single Shot",
        "price": 3.50,
        "rating": 4.5
    });

    let response = server.post("/api/coffees").json(&payload).await;

    let status = response.status_code();
    if status != StatusCode::CREATED {
        let body = response.text();
        eprintln!("Response status: {}", status);
        eprintln!("Response body: {}", body);
        panic!("Expected 201 CREATED, got {}", status);
    }

    assert_eq!(status, StatusCode::CREATED);
    
    let coffee: Coffee = response.json();
    assert!(coffee.id > 0, "Coffee should have a valid ID");
    assert_eq!(coffee.image_url, "https://images.unsplash.com/photo-1594146971821-373461fd5cd8");
    assert_eq!(coffee.name, "Espresso");
    assert_eq!(coffee.coffee_type, "Single Shot");
    assert_eq!(coffee.price, 3.50);
    assert_eq!(coffee.rating, 4.5);
}

/// Test coffee creation with zero price (invalid)
#[tokio::test]
async fn test_create_coffee_zero_price() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    let payload = json!({
        "image_url": "https://images.unsplash.com/photo-test",
        "name": "Invalid Coffee",
        "coffee_type": "Test",
        "price": 0.0,
        "rating": 4.5
    });

    let response = server.post("/api/coffees").json(&payload).await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json();
    assert_eq!(body["error_code"].as_str().unwrap(), "VALIDATION_ERROR");
    assert!(body["message"].as_str().unwrap().contains("validation failed"));
}

/// Test coffee creation with negative price (invalid)
#[tokio::test]
async fn test_create_coffee_negative_price() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    let payload = json!({
        "image_url": "https://images.unsplash.com/photo-test",
        "name": "Invalid Coffee",
        "coffee_type": "Test",
        "price": -1.50,
        "rating": 4.5
    });

    let response = server.post("/api/coffees").json(&payload).await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json();
    assert_eq!(body["error_code"].as_str().unwrap(), "VALIDATION_ERROR");
    assert!(body["message"].as_str().unwrap().contains("validation failed"));
}

/// Test coffee creation with rating below minimum (invalid)
#[tokio::test]
async fn test_create_coffee_rating_below_minimum() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    let payload = json!({
        "image_url": "https://images.unsplash.com/photo-test",
        "name": "Invalid Coffee",
        "coffee_type": "Test",
        "price": 3.50,
        "rating": -0.1
    });

    let response = server.post("/api/coffees").json(&payload).await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json();
    assert_eq!(body["error_code"].as_str().unwrap(), "VALIDATION_ERROR");
    assert!(body["message"].as_str().unwrap().contains("validation failed"));
}

/// Test coffee creation with rating above maximum (invalid)
#[tokio::test]
async fn test_create_coffee_rating_above_maximum() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    let payload = json!({
        "image_url": "https://images.unsplash.com/photo-test",
        "name": "Invalid Coffee",
        "coffee_type": "Test",
        "price": 3.50,
        "rating": 5.1
    });

    let response = server.post("/api/coffees").json(&payload).await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json();
    assert_eq!(body["error_code"].as_str().unwrap(), "VALIDATION_ERROR");
    assert!(body["message"].as_str().unwrap().contains("validation failed"));
}

/// Test coffee creation with valid boundary rating values
#[tokio::test]
async fn test_create_coffee_rating_boundaries() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Test minimum valid rating (0.0)
    let payload_min = json!({
        "image_url": "https://images.unsplash.com/photo-test",
        "name": "Min Rating Coffee",
        "coffee_type": "Test",
        "price": 3.50,
        "rating": 0.0
    });

    let response_min = server.post("/api/coffees").json(&payload_min).await;
    assert_eq!(response_min.status_code(), StatusCode::CREATED);

    // Test maximum valid rating (5.0)
    let payload_max = json!({
        "image_url": "https://images.unsplash.com/photo-test",
        "name": "Max Rating Coffee",
        "coffee_type": "Test",
        "price": 3.50,
        "rating": 5.0
    });

    let response_max = server.post("/api/coffees").json(&payload_max).await;
    assert_eq!(response_max.status_code(), StatusCode::CREATED);
}

// ============================================================================
// GET All Coffees Tests (GET /api/coffees)
// ============================================================================

/// Test retrieving all coffees when database is empty
#[tokio::test]
async fn test_get_all_coffees_empty() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    let response = server.get("/api/coffees").await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let coffees: Vec<Coffee> = response.json();
    assert_eq!(coffees.len(), 0);
}

/// Test retrieving all coffees with multiple items
#[tokio::test]
async fn test_get_all_coffees_multiple() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Create three coffees
    let coffees_to_create = vec!["Espresso", "Latte", "Cappuccino"];
    
    for name in &coffees_to_create {
        let payload = create_valid_coffee_payload(name);
        server.post("/api/coffees").json(&payload).await;
    }

    // Get all coffees
    let response = server.get("/api/coffees").await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let coffees: Vec<Coffee> = response.json();
    assert_eq!(coffees.len(), 3);
    
    // Verify names match
    let names: Vec<String> = coffees.iter().map(|c| c.name.clone()).collect();
    for name in coffees_to_create {
        assert!(names.contains(&name.to_string()));
    }
}

// ============================================================================
// GET Coffee by ID Tests (GET /api/coffees/:id)
// ============================================================================

/// Test retrieving a specific coffee by valid ID
#[tokio::test]
async fn test_get_coffee_by_id_success() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Create a coffee
    let payload = create_valid_coffee_payload("Cappuccino");
    let create_response = server.post("/api/coffees").json(&payload).await;
    let created_coffee: Coffee = create_response.json();

    // Get the coffee by ID
    let response = server.get(&format!("/api/coffees/{}", created_coffee.id)).await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let coffee: Coffee = response.json();
    assert_eq!(coffee.id, created_coffee.id);
    assert_eq!(coffee.name, "Cappuccino");
}

/// Test retrieving a non-existent coffee by ID
#[tokio::test]
async fn test_get_coffee_by_id_not_found() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    let response = server.get("/api/coffees/99999").await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    let body: serde_json::Value = response.json();
    assert_eq!(body["error_code"].as_str().unwrap(), "NOT_FOUND");
    assert!(body["message"].as_str().unwrap().contains("not found"));
}

// ============================================================================
// UPDATE Coffee Tests (PUT /api/coffees/:id)
// ============================================================================

/// Test updating a coffee with all fields
#[tokio::test]
async fn test_update_coffee_all_fields() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Create a coffee
    let payload = create_valid_coffee_payload("Original Name");
    let create_response = server.post("/api/coffees").json(&payload).await;
    let created_coffee: Coffee = create_response.json();

    // Update all fields
    let update_payload = json!({
        "image_url": "https://images.unsplash.com/photo-updated",
        "name": "Updated Name",
        "coffee_type": "Updated Type",
        "price": 5.00,
        "rating": 5.0
    });

    let response = server.put(&format!("/api/coffees/{}", created_coffee.id))
        .json(&update_payload)
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let updated_coffee: Coffee = response.json();
    assert_eq!(updated_coffee.image_url, "https://images.unsplash.com/photo-updated");
    assert_eq!(updated_coffee.name, "Updated Name");
    assert_eq!(updated_coffee.coffee_type, "Updated Type");
    assert_eq!(updated_coffee.price, 5.00);
    assert_eq!(updated_coffee.rating, 5.0);
}

/// Test updating a coffee with partial fields
#[tokio::test]
async fn test_update_coffee_partial_fields() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Create a coffee
    let payload = create_valid_coffee_payload("Original Name");
    let create_response = server.post("/api/coffees").json(&payload).await;
    let created_coffee: Coffee = create_response.json();

    // Update only name and price
    let update_payload = json!({
        "name": "Partially Updated",
        "price": 4.50
    });

    let response = server.put(&format!("/api/coffees/{}", created_coffee.id))
        .json(&update_payload)
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let updated_coffee: Coffee = response.json();
    assert_eq!(updated_coffee.name, "Partially Updated");
    assert_eq!(updated_coffee.price, 4.50);
    // Other fields should remain unchanged
    assert_eq!(updated_coffee.coffee_type, created_coffee.coffee_type);
    assert_eq!(updated_coffee.rating, created_coffee.rating);
}

/// Test updating a non-existent coffee
#[tokio::test]
async fn test_update_coffee_not_found() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    let update_payload = json!({
        "name": "Updated Name"
    });

    let response = server.put("/api/coffees/99999").json(&update_payload).await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    let body: serde_json::Value = response.json();
    assert_eq!(body["error_code"].as_str().unwrap(), "NOT_FOUND");
    assert!(body["message"].as_str().unwrap().contains("not found"));
}

/// Test updating with invalid price (zero)
#[tokio::test]
async fn test_update_coffee_invalid_price_zero() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Create a coffee
    let payload = create_valid_coffee_payload("Test Coffee");
    let create_response = server.post("/api/coffees").json(&payload).await;
    let created_coffee: Coffee = create_response.json();

    // Try to update with zero price
    let update_payload = json!({
        "price": 0.0
    });

    let response = server.put(&format!("/api/coffees/{}", created_coffee.id))
        .json(&update_payload)
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json();
    assert_eq!(body["error_code"].as_str().unwrap(), "VALIDATION_ERROR");
    assert!(body["message"].as_str().unwrap().contains("validation failed"));
}

/// Test updating with invalid price (negative)
#[tokio::test]
async fn test_update_coffee_invalid_price_negative() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Create a coffee
    let payload = create_valid_coffee_payload("Test Coffee");
    let create_response = server.post("/api/coffees").json(&payload).await;
    let created_coffee: Coffee = create_response.json();

    // Try to update with negative price
    let update_payload = json!({
        "price": -0.50
    });

    let response = server.put(&format!("/api/coffees/{}", created_coffee.id))
        .json(&update_payload)
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json();
    assert_eq!(body["error_code"].as_str().unwrap(), "VALIDATION_ERROR");
    assert!(body["message"].as_str().unwrap().contains("validation failed"));
}

/// Test updating with invalid rating (below minimum)
#[tokio::test]
async fn test_update_coffee_invalid_rating_below() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Create a coffee
    let payload = create_valid_coffee_payload("Test Coffee");
    let create_response = server.post("/api/coffees").json(&payload).await;
    let created_coffee: Coffee = create_response.json();

    // Try to update with rating below 0.0
    let update_payload = json!({
        "rating": -0.5
    });

    let response = server.put(&format!("/api/coffees/{}", created_coffee.id))
        .json(&update_payload)
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json();
    assert_eq!(body["error_code"].as_str().unwrap(), "VALIDATION_ERROR");
    assert!(body["message"].as_str().unwrap().contains("validation failed"));
}

/// Test updating with invalid rating (above maximum)
#[tokio::test]
async fn test_update_coffee_invalid_rating_above() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Create a coffee
    let payload = create_valid_coffee_payload("Test Coffee");
    let create_response = server.post("/api/coffees").json(&payload).await;
    let created_coffee: Coffee = create_response.json();

    // Try to update with rating above 5.0
    let update_payload = json!({
        "rating": 6.0
    });

    let response = server.put(&format!("/api/coffees/{}", created_coffee.id))
        .json(&update_payload)
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json();
    assert_eq!(body["error_code"].as_str().unwrap(), "VALIDATION_ERROR");
    assert!(body["message"].as_str().unwrap().contains("validation failed"));
}

// ============================================================================
// DELETE Coffee Tests (DELETE /api/coffees/:id)
// ============================================================================

/// Test deleting a coffee successfully
#[tokio::test]
async fn test_delete_coffee_success() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Create a coffee
    let payload = create_valid_coffee_payload("To Be Deleted");
    let create_response = server.post("/api/coffees").json(&payload).await;
    let created_coffee: Coffee = create_response.json();

    // Delete the coffee
    let response = server.delete(&format!("/api/coffees/{}", created_coffee.id)).await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    // Verify it's deleted by trying to get it
    let get_response = server.get(&format!("/api/coffees/{}", created_coffee.id)).await;
    assert_eq!(get_response.status_code(), StatusCode::NOT_FOUND);
}

/// Test deleting a non-existent coffee
#[tokio::test]
async fn test_delete_coffee_not_found() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    let response = server.delete("/api/coffees/99999").await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    let body: serde_json::Value = response.json();
    assert_eq!(body["error_code"].as_str().unwrap(), "NOT_FOUND");
    assert!(body["message"].as_str().unwrap().contains("not found"));
}

/// Test deleting a coffee twice (idempotency check)
#[tokio::test]
async fn test_delete_coffee_twice() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Create a coffee
    let payload = create_valid_coffee_payload("To Be Deleted Twice");
    let create_response = server.post("/api/coffees").json(&payload).await;
    let created_coffee: Coffee = create_response.json();

    // Delete the coffee first time
    let response1 = server.delete(&format!("/api/coffees/{}", created_coffee.id)).await;
    assert_eq!(response1.status_code(), StatusCode::NO_CONTENT);

    // Try to delete again
    let response2 = server.delete(&format!("/api/coffees/{}", created_coffee.id)).await;
    assert_eq!(response2.status_code(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Error Response Format Tests
// ============================================================================

/// Test that error responses have correct JSON format
#[tokio::test]
async fn test_error_response_format() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Trigger a 404 error
    let response = server.get("/api/coffees/99999").await;
    
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    let body: serde_json::Value = response.json();
    
    // Verify new error response format
    assert!(body.get("error_code").is_some());
    assert!(body["error_code"].is_string());
    assert_eq!(body["error_code"].as_str().unwrap(), "NOT_FOUND");
    
    assert!(body.get("message").is_some());
    assert!(body["message"].is_string());
    assert!(!body["message"].as_str().unwrap().is_empty());
    
    assert!(body.get("timestamp").is_some());
    assert!(body["timestamp"].is_string());
}

// ============================================================================
// Duplicate Detection Tests
// ============================================================================

/// Test creating a coffee with a duplicate name
#[tokio::test]
async fn test_create_coffee_duplicate_name() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Create first coffee
    let payload = create_valid_coffee_payload("Espresso");
    let response1 = server.post("/api/coffees").json(&payload).await;
    assert_eq!(response1.status_code(), StatusCode::CREATED);

    // Try to create another coffee with the same name
    let response2 = server.post("/api/coffees").json(&payload).await;
    
    assert_eq!(response2.status_code(), StatusCode::CONFLICT);
    let body: serde_json::Value = response2.json();
    assert_eq!(body["error_code"].as_str().unwrap(), "CONFLICT");
    assert!(body["message"].as_str().unwrap().contains("already exists"));
    assert!(body["message"].as_str().unwrap().contains("Espresso"));
}

/// Test creating coffees with different names (no conflict)
#[tokio::test]
async fn test_create_coffee_different_names_no_conflict() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Create first coffee
    let payload1 = create_valid_coffee_payload("Espresso");
    let response1 = server.post("/api/coffees").json(&payload1).await;
    assert_eq!(response1.status_code(), StatusCode::CREATED);

    // Create second coffee with different name
    let payload2 = create_valid_coffee_payload("Latte");
    let response2 = server.post("/api/coffees").json(&payload2).await;
    assert_eq!(response2.status_code(), StatusCode::CREATED);
}

/// Test updating a coffee to a duplicate name
#[tokio::test]
async fn test_update_coffee_duplicate_name() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Create two coffees with different names
    let payload1 = create_valid_coffee_payload("Espresso");
    let response1 = server.post("/api/coffees").json(&payload1).await;
    let _coffee1: Coffee = response1.json();

    let payload2 = create_valid_coffee_payload("Latte");
    let response2 = server.post("/api/coffees").json(&payload2).await;
    let coffee2: Coffee = response2.json();

    // Try to update coffee2 to have the same name as coffee1
    let update_payload = json!({
        "name": "Espresso"
    });

    let response = server.put(&format!("/api/coffees/{}", coffee2.id))
        .json(&update_payload)
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
    let body: serde_json::Value = response.json();
    assert_eq!(body["error_code"].as_str().unwrap(), "CONFLICT");
    assert!(body["message"].as_str().unwrap().contains("already exists"));
    assert!(body["message"].as_str().unwrap().contains("Espresso"));
}

/// Test updating a coffee to keep the same name (should succeed)
#[tokio::test]
async fn test_update_coffee_same_name_no_conflict() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Create a coffee
    let payload = create_valid_coffee_payload("Espresso");
    let create_response = server.post("/api/coffees").json(&payload).await;
    let created_coffee: Coffee = create_response.json();

    // Update the coffee but keep the same name
    let update_payload = json!({
        "name": "Espresso",
        "price": 4.00
    });

    let response = server.put(&format!("/api/coffees/{}", created_coffee.id))
        .json(&update_payload)
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let updated_coffee: Coffee = response.json();
    assert_eq!(updated_coffee.name, "Espresso");
    assert_eq!(updated_coffee.price, 4.00);
}

/// Test updating a coffee to a new unique name (should succeed)
#[tokio::test]
async fn test_update_coffee_new_unique_name() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Create two coffees
    let payload1 = create_valid_coffee_payload("Espresso");
    server.post("/api/coffees").json(&payload1).await;

    let payload2 = create_valid_coffee_payload("Latte");
    let response2 = server.post("/api/coffees").json(&payload2).await;
    let coffee2: Coffee = response2.json();

    // Update coffee2 to a new unique name
    let update_payload = json!({
        "name": "Cappuccino"
    });

    let response = server.put(&format!("/api/coffees/{}", coffee2.id))
        .json(&update_payload)
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let updated_coffee: Coffee = response.json();
    assert_eq!(updated_coffee.name, "Cappuccino");
}

// ============================================================================
// Transaction Rollback Tests
// ============================================================================

/// Test that update operation rolls back on duplicate name conflict
/// This verifies that when a duplicate name is detected during an update,
/// the transaction is rolled back and no partial changes are committed
#[tokio::test]
async fn test_update_coffee_rollback_on_duplicate() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Create two coffees
    let payload1 = create_valid_coffee_payload("Espresso");
    server.post("/api/coffees").json(&payload1).await;

    let payload2 = create_valid_coffee_payload("Latte");
    let response2 = server.post("/api/coffees").json(&payload2).await;
    let coffee2: Coffee = response2.json();
    let original_price = coffee2.price;

    // Try to update coffee2 with a duplicate name and new price
    // This should fail and rollback, leaving the original data intact
    let update_payload = json!({
        "name": "Espresso",  // Duplicate name - should cause conflict
        "price": 99.99       // This should NOT be saved due to rollback
    });

    let response = server.put(&format!("/api/coffees/{}", coffee2.id))
        .json(&update_payload)
        .await;

    // Verify the update failed with conflict
    assert_eq!(response.status_code(), StatusCode::CONFLICT);

    // Verify the coffee data was NOT changed (transaction rolled back)
    let get_response = server.get(&format!("/api/coffees/{}", coffee2.id)).await;
    assert_eq!(get_response.status_code(), StatusCode::OK);
    let unchanged_coffee: Coffee = get_response.json();
    
    // Original name should be preserved
    assert_eq!(unchanged_coffee.name, "Latte");
    // Original price should be preserved (not 99.99)
    assert_eq!(unchanged_coffee.price, original_price);
}

/// Test that update operation rolls back when coffee doesn't exist
/// This verifies that when a coffee is not found during an update,
/// the transaction is rolled back and no partial changes are committed
#[tokio::test]
async fn test_update_coffee_rollback_on_not_found() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Try to update a non-existent coffee
    let update_payload = json!({
        "name": "Should Not Be Created",
        "price": 99.99
    });

    let response = server.put("/api/coffees/99999")
        .json(&update_payload)
        .await;

    // Verify the update failed with not found
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);

    // Verify no coffee was created as a side effect
    let get_all_response = server.get("/api/coffees").await;
    let all_coffees: Vec<Coffee> = get_all_response.json();
    
    // Should still be empty (no partial data created)
    assert_eq!(all_coffees.len(), 0);
}

/// Test transaction-based helper function for price updates
/// This verifies that the transaction helper function works correctly
#[tokio::test]
async fn test_transaction_helper_success() {
    let pool = create_test_pool().await;
    clean_test_data(&pool).await;

    // Create a coffee directly in the database
    let coffee = sqlx::query_as::<_, Coffee>(
        r#"
        INSERT INTO coffees (image_url, name, coffee_type, price, rating)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, image_url, name, coffee_type, price, rating
        "#,
    )
    .bind("https://images.unsplash.com/photo-test")
    .bind("Test Coffee")
    .bind("Test Type")
    .bind(3.50)
    .bind(4.5)
    .fetch_one(&pool)
    .await
    .expect("Failed to create test coffee");

    // Use the transaction helper to update the price
    let result = crate::db::update_coffee_price_with_transaction(&pool, coffee.id, 5.99).await;
    assert!(result.is_ok(), "Transaction should succeed");

    // Verify the price was updated
    let updated = sqlx::query_as::<_, Coffee>(
        "SELECT id, image_url, name, coffee_type, price, rating FROM coffees WHERE id = $1"
    )
    .bind(coffee.id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch updated coffee");

    assert_eq!(updated.price, 5.99);
}

/// Test transaction-based helper function rolls back on not found
/// This verifies that the transaction helper function properly rolls back
/// when the coffee doesn't exist
#[tokio::test]
async fn test_transaction_helper_rollback_not_found() {
    let pool = create_test_pool().await;
    clean_test_data(&pool).await;

    // Verify database is empty before test
    let count_before: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM coffees")
        .fetch_one(&pool)
        .await
        .expect("Failed to count coffees");
    
    // Try to update a non-existent coffee
    let result = crate::db::update_coffee_price_with_transaction(&pool, 99999, 5.99).await;
    
    // Should return NotFound error
    assert!(result.is_err(), "Transaction should fail");
    match result {
        Err(ApiError::NotFound { resource, id }) => {
            assert_eq!(resource, "Coffee");
            assert_eq!(id, "99999");
        }
        _ => panic!("Expected NotFound error"),
    }

    // Verify no data was created or modified (count should be same as before)
    let count_after: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM coffees")
        .fetch_one(&pool)
        .await
        .expect("Failed to count coffees");
    
    assert_eq!(count_after.0, count_before.0, "No data should be created or modified after rollback");
}


// ============================================================================
// Task 9: Integration Testing and Final Validation
// ============================================================================

/// Helper function to register a user and return auth tokens
async fn register_user(server: &TestServer, email: &str, password: &str) -> serde_json::Value {
    let payload = json!({
        "email": email,
        "password": password
    });
    
    let response = server.post("/api/auth/register").json(&payload).await;
    assert_eq!(response.status_code(), StatusCode::OK, "User registration failed");
    response.json()
}

/// Helper function to create an admin user directly in the database
async fn create_admin_user(pool: &PgPool, email: &str, password: &str) -> i32 {
    let password_hash = crate::auth::password::PasswordService::hash_password(password)
        .expect("Failed to hash password");
    
    let user_id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO users (email, password_hash, role)
        VALUES ($1, $2, 'admin')
        RETURNING id
        "#
    )
    .bind(email)
    .bind(password_hash)
    .fetch_one(pool)
    .await
    .expect("Failed to create admin user");
    
    user_id
}

/// Helper function to login and get auth tokens
async fn login_user(server: &TestServer, email: &str, password: &str) -> serde_json::Value {
    let payload = json!({
        "email": email,
        "password": password
    });
    
    let response = server.post("/api/auth/login").json(&payload).await;
    
    if response.status_code() != StatusCode::OK {
        let error_body = response.text();
        eprintln!("Login failed with status {}: {}", response.status_code(), error_body);
        panic!("User login failed");
    }
    
    response.json()
}

/// Helper function to clean auth test data
async fn clean_auth_test_data(pool: &PgPool) {
    sqlx::query("TRUNCATE TABLE refresh_tokens RESTART IDENTITY CASCADE")
        .execute(pool)
        .await
        .expect("Failed to clean refresh_tokens");
    
    sqlx::query("TRUNCATE TABLE users RESTART IDENTITY CASCADE")
        .execute(pool)
        .await
        .expect("Failed to clean users");
    
    sqlx::query("TRUNCATE TABLE coffees RESTART IDENTITY CASCADE")
        .execute(pool)
        .await
        .expect("Failed to clean coffees");
}

// ============================================================================
// Task 9.1: End-to-End Authorization Flow Integration Tests
// ============================================================================

/// Test complete request flow from client to protected route
/// Validates: Requirements 6.1, 6.2
#[tokio::test]
async fn test_e2e_authorization_flow_with_token() {
    // Set JWT_SECRET for middleware
    std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");
    
    let pool = create_test_pool().await;
    clean_auth_test_data(&pool).await;
    
    // Create admin user directly in database
    let user_id = create_admin_user(&pool, "admin@test.com", "adminpass123").await;
    eprintln!("Created admin user with id: {}", user_id);
    
    // Verify user exists
    let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = 'admin@test.com'")
        .fetch_one(&pool)
        .await
        .expect("Failed to count users");
    eprintln!("User count: {}", user_count.0);
    
    let server = create_test_app(pool).await;
    
    // Step 1: Login as admin to get token
    let auth_response = login_user(&server, "admin@test.com", "adminpass123").await;
    let access_token = auth_response["access_token"].as_str().unwrap();
    
    // Step 2: Use token to access protected route (create coffee)
    let coffee_payload = json!({
        "image_url": "https://images.unsplash.com/photo-test",
        "name": "Admin Created Coffee",
        "coffee_type": "Espresso",
        "price": 4.50,
        "rating": 4.8
    });
    
    let response = server
        .post("/api/coffees")
        .add_header("Authorization".parse().unwrap(), format!("Bearer {}", access_token).parse().unwrap())
        .json(&coffee_payload)
        .await;
    
    assert_eq!(response.status_code(), StatusCode::CREATED);
    let created_coffee: Coffee = response.json();
    assert_eq!(created_coffee.name, "Admin Created Coffee");
    
    // Step 3: Verify coffee was created by fetching it
    let get_response = server.get(&format!("/api/coffees/{}", created_coffee.id)).await;
    assert_eq!(get_response.status_code(), StatusCode::OK);
    let fetched_coffee: Coffee = get_response.json();
    assert_eq!(fetched_coffee.name, "Admin Created Coffee");
}

/// Test token generation with role through to route access
/// Validates: Requirements 6.1, 6.2
#[tokio::test]
async fn test_token_contains_role_and_grants_access() {
    let pool = create_test_pool().await;
    clean_auth_test_data(&pool).await;
    
    // Create admin user
    create_admin_user(&pool, "admin@test.com", "adminpass123").await;
    
    let server = create_test_app(pool).await;
    
    // Login and get token
    let auth_response = login_user(&server, "admin@test.com", "adminpass123").await;
    let access_token = auth_response["access_token"].as_str().unwrap();
    
    // Use token to create coffee (admin permission required)
    let coffee_payload = json!({
        "image_url": "https://images.unsplash.com/photo-test",
        "name": "Role Test Coffee",
        "coffee_type": "Espresso",
        "price": 4.50,
        "rating": 4.8
    });
    
    let response = server
        .post("/api/coffees")
        .add_header("Authorization".parse().unwrap(), format!("Bearer {}", access_token).parse().unwrap())
        .json(&coffee_payload)
        .await;
    
    // Admin role should grant access
    assert_eq!(response.status_code(), StatusCode::CREATED);
}

/// Test role updates reflected in new tokens
/// Validates: Requirements 6.5
#[tokio::test]
async fn test_role_updates_reflected_in_new_tokens() {
    let pool = create_test_pool().await;
    clean_auth_test_data(&pool).await;
    
    // Create regular user
    let password_hash = crate::auth::password::PasswordService::hash_password("userpass123").expect("Failed to hash password");
    
    let user_id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO users (email, password_hash, role)
        VALUES ($1, $2, 'user')
        RETURNING id
        "#
    )
    .bind("user@test.com")
    .bind(password_hash)
    .fetch_one(&pool)
    .await
    .expect("Failed to create user");
    
    let server = create_test_app(pool.clone()).await;
    
    // Login as regular user
    let auth_response = login_user(&server, "user@test.com", "userpass123").await;
    let access_token = auth_response["access_token"].as_str().unwrap();
    
    // Try to create coffee (should fail - user role)
    let coffee_payload = json!({
        "image_url": "https://images.unsplash.com/photo-test",
        "name": "Should Fail Coffee",
        "coffee_type": "Espresso",
        "price": 4.50,
        "rating": 4.8
    });
    
    let response = server
        .post("/api/coffees")
        .add_header("Authorization".parse().unwrap(), format!("Bearer {}", access_token).parse().unwrap())
        .json(&coffee_payload)
        .await;
    
    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
    
    // Update user role to admin
    sqlx::query("UPDATE users SET role = 'admin' WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("Failed to update user role");
    
    // Login again to get new token with updated role
    let new_auth_response = login_user(&server, "user@test.com", "userpass123").await;
    let new_access_token = new_auth_response["access_token"].as_str().unwrap();
    
    // Try to create coffee again (should succeed - admin role)
    let coffee_payload2 = json!({
        "image_url": "https://images.unsplash.com/photo-test",
        "name": "Should Succeed Coffee",
        "coffee_type": "Espresso",
        "price": 4.50,
        "rating": 4.8
    });
    
    let response2 = server
        .post("/api/coffees")
        .add_header("Authorization".parse().unwrap(), format!("Bearer {}", new_access_token).parse().unwrap())
        .json(&coffee_payload2)
        .await;
    
    assert_eq!(response2.status_code(), StatusCode::CREATED);
}

// ============================================================================
// Task 9.2: Coffee Route Protection Integration Tests
// ============================================================================

/// Test admin can create, update, delete coffee
/// Validates: Requirements 2.1, 2.2, 2.3
#[tokio::test]
async fn test_admin_can_manage_coffee() {
    let pool = create_test_pool().await;
    clean_auth_test_data(&pool).await;
    
    // Create admin user
    create_admin_user(&pool, "admin@test.com", "adminpass123").await;
    
    let server = create_test_app(pool).await;
    
    // Login as admin
    let auth_response = login_user(&server, "admin@test.com", "adminpass123").await;
    let access_token = auth_response["access_token"].as_str().unwrap();
    
    // Test CREATE
    let create_payload = json!({
        "image_url": "https://images.unsplash.com/photo-test",
        "name": "Admin Coffee",
        "coffee_type": "Espresso",
        "price": 4.50,
        "rating": 4.8
    });
    
    let create_response = server
        .post("/api/coffees")
        .add_header("Authorization".parse().unwrap(), format!("Bearer {}", access_token).parse().unwrap())
        .json(&create_payload)
        .await;
    
    assert_eq!(create_response.status_code(), StatusCode::CREATED);
    let created_coffee: Coffee = create_response.json();
    
    // Test UPDATE
    let update_payload = json!({
        "name": "Updated Admin Coffee",
        "price": 5.00
    });
    
    let update_response = server
        .put(&format!("/api/coffees/{}", created_coffee.id))
        .add_header("Authorization".parse().unwrap(), format!("Bearer {}", access_token).parse().unwrap())
        .json(&update_payload)
        .await;
    
    assert_eq!(update_response.status_code(), StatusCode::OK);
    let updated_coffee: Coffee = update_response.json();
    assert_eq!(updated_coffee.name, "Updated Admin Coffee");
    assert_eq!(updated_coffee.price, 5.00);
    
    // Test DELETE
    let delete_response = server
        .delete(&format!("/api/coffees/{}", created_coffee.id))
        .add_header("Authorization".parse().unwrap(), format!("Bearer {}", access_token).parse().unwrap())
        .await;
    
    assert_eq!(delete_response.status_code(), StatusCode::NO_CONTENT);
}

/// Test regular user cannot create, update, delete coffee
/// Validates: Requirements 2.4, 2.5, 2.6
#[tokio::test]
async fn test_regular_user_cannot_manage_coffee() {
    let pool = create_test_pool().await;
    clean_auth_test_data(&pool).await;
    
    // Create admin user to set up test data
    create_admin_user(&pool, "admin@test.com", "adminpass123").await;
    
    // Create regular user
    
    let password_hash = crate::auth::password::PasswordService::hash_password("userpass123").expect("Failed to hash password");
    
    sqlx::query(
        r#"
        INSERT INTO users (email, password_hash, role)
        VALUES ($1, $2, 'user')
        "#
    )
    .bind("user@test.com")
    .bind(password_hash)
    .execute(&pool)
    .await
    .expect("Failed to create user");
    
    let server = create_test_app(pool.clone()).await;
    
    // Login as admin to create a coffee for testing
    let admin_auth = login_user(&server, "admin@test.com", "adminpass123").await;
    let admin_token = admin_auth["access_token"].as_str().unwrap();
    
    let coffee_payload = json!({
        "image_url": "https://images.unsplash.com/photo-test",
        "name": "Test Coffee",
        "coffee_type": "Espresso",
        "price": 4.50,
        "rating": 4.8
    });
    
    let create_response = server
        .post("/api/coffees")
        .add_header("Authorization".parse().unwrap(), format!("Bearer {}", admin_token).parse().unwrap())
        .json(&coffee_payload)
        .await;
    
    let created_coffee: Coffee = create_response.json();
    
    // Login as regular user
    let user_auth = login_user(&server, "user@test.com", "userpass123").await;
    let user_token = user_auth["access_token"].as_str().unwrap();
    
    // Test CREATE (should fail)
    let create_payload = json!({
        "image_url": "https://images.unsplash.com/photo-test",
        "name": "User Coffee",
        "coffee_type": "Latte",
        "price": 3.50,
        "rating": 4.0
    });
    
    let create_response = server
        .post("/api/coffees")
        .add_header("Authorization".parse().unwrap(), format!("Bearer {}", user_token).parse().unwrap())
        .json(&create_payload)
        .await;
    
    assert_eq!(create_response.status_code(), StatusCode::FORBIDDEN);
    
    // Test UPDATE (should fail)
    let update_payload = json!({
        "name": "User Updated Coffee"
    });
    
    let update_response = server
        .put(&format!("/api/coffees/{}", created_coffee.id))
        .add_header("Authorization".parse().unwrap(), format!("Bearer {}", user_token).parse().unwrap())
        .json(&update_payload)
        .await;
    
    assert_eq!(update_response.status_code(), StatusCode::FORBIDDEN);
    
    // Test DELETE (should fail)
    let delete_response = server
        .delete(&format!("/api/coffees/{}", created_coffee.id))
        .add_header("Authorization".parse().unwrap(), format!("Bearer {}", user_token).parse().unwrap())
        .await;
    
    assert_eq!(delete_response.status_code(), StatusCode::FORBIDDEN);
}

/// Test both roles can list and view coffee
/// Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 2.6
#[tokio::test]
async fn test_both_roles_can_view_coffee() {
    let pool = create_test_pool().await;
    clean_auth_test_data(&pool).await;
    
    // Create admin user
    create_admin_user(&pool, "admin@test.com", "adminpass123").await;
    
    // Create regular user
    
    let password_hash = crate::auth::password::PasswordService::hash_password("userpass123").expect("Failed to hash password");
    
    sqlx::query(
        r#"
        INSERT INTO users (email, password_hash, role)
        VALUES ($1, $2, 'user')
        "#
    )
    .bind("user@test.com")
    .bind(password_hash)
    .execute(&pool)
    .await
    .expect("Failed to create user");
    
    let server = create_test_app(pool.clone()).await;
    
    // Login as admin and create a coffee
    let admin_auth = login_user(&server, "admin@test.com", "adminpass123").await;
    let admin_token = admin_auth["access_token"].as_str().unwrap();
    
    let coffee_payload = json!({
        "image_url": "https://images.unsplash.com/photo-test",
        "name": "Public Coffee",
        "coffee_type": "Espresso",
        "price": 4.50,
        "rating": 4.8
    });
    
    let create_response = server
        .post("/api/coffees")
        .add_header("Authorization".parse().unwrap(), format!("Bearer {}", admin_token).parse().unwrap())
        .json(&coffee_payload)
        .await;
    
    let created_coffee: Coffee = create_response.json();
    
    // Login as regular user
    let user_auth = login_user(&server, "user@test.com", "userpass123").await;
    let user_token = user_auth["access_token"].as_str().unwrap();
    
    // Test LIST as admin (no auth required, but test with token)
    let list_response_admin = server
        .get("/api/coffees")
        .add_header("Authorization".parse().unwrap(), format!("Bearer {}", admin_token).parse().unwrap())
        .await;
    
    assert_eq!(list_response_admin.status_code(), StatusCode::OK);
    let coffees_admin: Vec<Coffee> = list_response_admin.json();
    assert!(coffees_admin.len() > 0);
    
    // Test LIST as user (no auth required, but test with token)
    let list_response_user = server
        .get("/api/coffees")
        .add_header("Authorization".parse().unwrap(), format!("Bearer {}", user_token).parse().unwrap())
        .await;
    
    assert_eq!(list_response_user.status_code(), StatusCode::OK);
    let coffees_user: Vec<Coffee> = list_response_user.json();
    assert!(coffees_user.len() > 0);
    
    // Test GET by ID as admin (no auth required, but test with token)
    let get_response_admin = server
        .get(&format!("/api/coffees/{}", created_coffee.id))
        .add_header("Authorization".parse().unwrap(), format!("Bearer {}", admin_token).parse().unwrap())
        .await;
    
    assert_eq!(get_response_admin.status_code(), StatusCode::OK);
    let coffee_admin: Coffee = get_response_admin.json();
    assert_eq!(coffee_admin.name, "Public Coffee");
    
    // Test GET by ID as user (no auth required, but test with token)
    let get_response_user = server
        .get(&format!("/api/coffees/{}", created_coffee.id))
        .add_header("Authorization".parse().unwrap(), format!("Bearer {}", user_token).parse().unwrap())
        .await;
    
    assert_eq!(get_response_user.status_code(), StatusCode::OK);
    let coffee_user: Coffee = get_response_user.json();
    assert_eq!(coffee_user.name, "Public Coffee");
    
    // Test LIST without auth (should also work - public endpoint)
    let list_response_no_auth = server.get("/api/coffees").await;
    assert_eq!(list_response_no_auth.status_code(), StatusCode::OK);
    
    // Test GET by ID without auth (should also work - public endpoint)
    let get_response_no_auth = server.get(&format!("/api/coffees/{}", created_coffee.id)).await;
    assert_eq!(get_response_no_auth.status_code(), StatusCode::OK);
}

/// Test protected routes reject requests without tokens
/// Validates: Requirements 4.2
#[tokio::test]
async fn test_protected_routes_reject_no_token() {
    let pool = create_test_pool().await;
    clean_auth_test_data(&pool).await;
    
    let server = create_test_app(pool).await;
    
    // Test CREATE without token
    let create_payload = json!({
        "image_url": "https://images.unsplash.com/photo-test",
        "name": "No Token Coffee",
        "coffee_type": "Espresso",
        "price": 4.50,
        "rating": 4.8
    });
    
    let create_response = server
        .post("/api/coffees")
        .json(&create_payload)
        .await;
    
    assert_eq!(create_response.status_code(), StatusCode::UNAUTHORIZED);
    
    // Test UPDATE without token
    let update_payload = json!({
        "name": "Updated Name"
    });
    
    let update_response = server
        .put("/api/coffees/1")
        .json(&update_payload)
        .await;
    
    assert_eq!(update_response.status_code(), StatusCode::UNAUTHORIZED);
    
    // Test DELETE without token
    let delete_response = server
        .delete("/api/coffees/1")
        .await;
    
    assert_eq!(delete_response.status_code(), StatusCode::UNAUTHORIZED);
}

/// Test protected routes reject requests with invalid tokens
/// Validates: Requirements 4.2
#[tokio::test]
async fn test_protected_routes_reject_invalid_token() {
    let pool = create_test_pool().await;
    clean_auth_test_data(&pool).await;
    
    let server = create_test_app(pool).await;
    
    let invalid_token = "invalid.jwt.token";
    
    // Test CREATE with invalid token
    let create_payload = json!({
        "image_url": "https://images.unsplash.com/photo-test",
        "name": "Invalid Token Coffee",
        "coffee_type": "Espresso",
        "price": 4.50,
        "rating": 4.8
    });
    
    let create_response = server
        .post("/api/coffees")
        .add_header("Authorization".parse().unwrap(), format!("Bearer {}", invalid_token).parse().unwrap())
        .json(&create_payload)
        .await;
    
    assert_eq!(create_response.status_code(), StatusCode::UNAUTHORIZED);
    
    // Test UPDATE with invalid token
    let update_payload = json!({
        "name": "Updated Name"
    });
    
    let update_response = server
        .put("/api/coffees/1")
        .add_header("Authorization".parse().unwrap(), format!("Bearer {}", invalid_token).parse().unwrap())
        .json(&update_payload)
        .await;
    
    assert_eq!(update_response.status_code(), StatusCode::UNAUTHORIZED);
    
    // Test DELETE with invalid token
    let delete_response = server
        .delete("/api/coffees/1")
        .add_header("Authorization".parse().unwrap(), format!("Bearer {}", invalid_token).parse().unwrap())
        .await;
    
    assert_eq!(delete_response.status_code(), StatusCode::UNAUTHORIZED)
;
}



/// Debug test to check if user can be loaded from database
#[tokio::test]
async fn test_debug_user_loading() {
    let pool = create_test_pool().await;
    clean_auth_test_data(&pool).await;
    
    // Create admin user
    let user_id = create_admin_user(&pool, "debug@test.com", "debugpass123").await;
    eprintln!("Created user with id: {}", user_id);
    
    // Try to load the user
    let user_repo = crate::auth::repository::UserRepository::new(pool.clone());
    let user = user_repo.find_by_email("debug@test.com").await;
    
    match user {
        Ok(Some(u)) => {
            eprintln!("Successfully loaded user: id={}, email={}, role={:?}", u.id, u.email, u.role);
        }
        Ok(None) => {
            eprintln!("User not found!");
        }
        Err(e) => {
            eprintln!("Error loading user: {:?}", e);
        }
    }
}
