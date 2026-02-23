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
    // Clean test data to ensure isolation
    clean_test_data(&pool).await;
    
    let state = AppState { db: pool };
    
    let app = Router::new()
        .route("/api/coffees", post(create_coffee))
        .route("/api/coffees", get(get_all_coffees))
        .route("/api/coffees/:id", get(get_coffee_by_id))
        .route("/api/coffees/:id", put(update_coffee))
        .route("/api/coffees/:id", delete(delete_coffee))
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

