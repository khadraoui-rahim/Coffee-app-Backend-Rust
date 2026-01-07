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
/// Connects to the database, runs migrations, and cleans test data
async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://coffee_user:coffee_pass@db:5432/coffee_db".to_string());
    
    let pool = crate::db::create_pool(&database_url)
        .await
        .expect("Failed to connect to test database");
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    // Clean up any existing test data
    sqlx::query("DELETE FROM coffees")
        .execute(&pool)
        .await
        .expect("Failed to clean test data");
    
    pool
}

/// Helper function to create a test app with database
async fn create_test_app(pool: PgPool) -> TestServer {
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
        "name": name,
        "coffee_type": "Test Type",
        "price": 300,
        "rating": 4.5,
        "temperature": "hot",
        "description": "Test description",
        "size": "medium",
        "liked": false
    })
}

// ============================================================================
// CREATE Coffee Tests (POST /api/coffees)
// ============================================================================

/// Test successful coffee creation with all valid fields
/// Requirements: 2.1, 2.2, 2.4
#[tokio::test]
async fn test_create_coffee_success() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    let payload = json!({
        "name": "Espresso",
        "coffee_type": "Single Shot",
        "price": 250,
        "rating": 4.5,
        "temperature": "hot",
        "description": "Strong and bold",
        "size": "small",
        "liked": true
    });

    let response = server.post("/api/coffees").json(&payload).await;

    // Debug: print response if not successful
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
    assert_eq!(coffee.name, "Espresso");
    assert_eq!(coffee.coffee_type, "Single Shot");
    assert_eq!(coffee.price, 250);
    assert_eq!(coffee.rating, 4.5);
    assert_eq!(coffee.temperature, "hot");
    assert_eq!(coffee.description, "Strong and bold");
    assert_eq!(coffee.size, "small");
    assert_eq!(coffee.liked, true);
}

/// Test coffee creation with zero price (invalid)
/// Requirements: 2.3, 8.4
#[tokio::test]
async fn test_create_coffee_zero_price() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    let payload = json!({
        "name": "Invalid Coffee",
        "coffee_type": "Test",
        "price": 0,
        "rating": 4.5,
        "temperature": "hot",
        "description": "Test",
        "size": "medium",
        "liked": false
    });

    let response = server.post("/api/coffees").json(&payload).await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json();
    assert!(body["error"].as_str().unwrap().contains("Price must be a positive integer"));
}

/// Test coffee creation with negative price (invalid)
/// Requirements: 2.3, 8.4
#[tokio::test]
async fn test_create_coffee_negative_price() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    let payload = json!({
        "name": "Invalid Coffee",
        "coffee_type": "Test",
        "price": -100,
        "rating": 4.5,
        "temperature": "hot",
        "description": "Test",
        "size": "medium",
        "liked": false
    });

    let response = server.post("/api/coffees").json(&payload).await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json();
    assert!(body["error"].as_str().unwrap().contains("Price must be a positive integer"));
}

/// Test coffee creation with rating below minimum (invalid)
/// Requirements: 2.3, 8.5
#[tokio::test]
async fn test_create_coffee_rating_below_minimum() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    let payload = json!({
        "name": "Invalid Coffee",
        "coffee_type": "Test",
        "price": 300,
        "rating": -0.1,
        "temperature": "hot",
        "description": "Test",
        "size": "medium",
        "liked": false
    });

    let response = server.post("/api/coffees").json(&payload).await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json();
    assert!(body["error"].as_str().unwrap().contains("Rating must be between 0.0 and 5.0"));
}

/// Test coffee creation with rating above maximum (invalid)
/// Requirements: 2.3, 8.5
#[tokio::test]
async fn test_create_coffee_rating_above_maximum() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    let payload = json!({
        "name": "Invalid Coffee",
        "coffee_type": "Test",
        "price": 300,
        "rating": 5.1,
        "temperature": "hot",
        "description": "Test",
        "size": "medium",
        "liked": false
    });

    let response = server.post("/api/coffees").json(&payload).await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json();
    assert!(body["error"].as_str().unwrap().contains("Rating must be between 0.0 and 5.0"));
}

/// Test coffee creation with valid boundary rating values
/// Requirements: 2.1, 2.2, 8.5
#[tokio::test]
async fn test_create_coffee_rating_boundaries() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Test minimum valid rating (0.0)
    let payload_min = json!({
        "name": "Min Rating Coffee",
        "coffee_type": "Test",
        "price": 300,
        "rating": 0.0,
        "temperature": "hot",
        "description": "Test",
        "size": "medium",
        "liked": false
    });

    let response_min = server.post("/api/coffees").json(&payload_min).await;
    assert_eq!(response_min.status_code(), StatusCode::CREATED);

    // Test maximum valid rating (5.0)
    let payload_max = json!({
        "name": "Max Rating Coffee",
        "coffee_type": "Test",
        "price": 300,
        "rating": 5.0,
        "temperature": "hot",
        "description": "Test",
        "size": "medium",
        "liked": false
    });

    let response_max = server.post("/api/coffees").json(&payload_max).await;
    assert_eq!(response_max.status_code(), StatusCode::CREATED);
}

/// Test coffee creation with invalid temperature value
/// Requirements: 2.3, 8.6
#[tokio::test]
async fn test_create_coffee_invalid_temperature() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    let payload = json!({
        "name": "Invalid Coffee",
        "coffee_type": "Test",
        "price": 300,
        "rating": 4.5,
        "temperature": "warm",
        "description": "Test",
        "size": "medium",
        "liked": false
    });

    let response = server.post("/api/coffees").json(&payload).await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json();
    assert!(body["error"].as_str().unwrap().contains("Temperature must be one of: hot, cold, both"));
}

/// Test coffee creation with all valid temperature values
/// Requirements: 2.1, 2.2, 8.6
#[tokio::test]
async fn test_create_coffee_all_valid_temperatures() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    let temperatures = vec!["hot", "cold", "both"];

    for temp in temperatures {
        let payload = json!({
            "name": format!("{} Coffee", temp),
            "coffee_type": "Test",
            "price": 300,
            "rating": 4.5,
            "temperature": temp,
            "description": "Test",
            "size": "medium",
            "liked": false
        });

        let response = server.post("/api/coffees").json(&payload).await;
        assert_eq!(
            response.status_code(),
            StatusCode::CREATED,
            "Temperature '{}' should be valid",
            temp
        );
    }
}

// ============================================================================
// GET All Coffees Tests (GET /api/coffees)
// ============================================================================

/// Test retrieving all coffees when database is empty
/// Requirements: 3.1, 3.4
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
/// Requirements: 3.1, 3.4
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
/// Requirements: 3.2, 3.4
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
/// Requirements: 3.3, 8.3
#[tokio::test]
async fn test_get_coffee_by_id_not_found() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    let response = server.get("/api/coffees/99999").await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    let body: serde_json::Value = response.json();
    assert!(body["error"].as_str().unwrap().contains("not found"));
}

// ============================================================================
// UPDATE Coffee Tests (PUT /api/coffees/:id)
// ============================================================================

/// Test updating a coffee with all fields
/// Requirements: 4.1, 4.2
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
        "name": "Updated Name",
        "coffee_type": "Updated Type",
        "price": 500,
        "rating": 5.0,
        "temperature": "cold",
        "description": "Updated description",
        "size": "large",
        "liked": true
    });

    let response = server.put(&format!("/api/coffees/{}", created_coffee.id))
        .json(&update_payload)
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let updated_coffee: Coffee = response.json();
    assert_eq!(updated_coffee.name, "Updated Name");
    assert_eq!(updated_coffee.coffee_type, "Updated Type");
    assert_eq!(updated_coffee.price, 500);
    assert_eq!(updated_coffee.rating, 5.0);
    assert_eq!(updated_coffee.temperature, "cold");
    assert_eq!(updated_coffee.description, "Updated description");
    assert_eq!(updated_coffee.size, "large");
    assert_eq!(updated_coffee.liked, true);
}

/// Test updating a coffee with partial fields
/// Requirements: 4.1, 4.2
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
        "price": 450
    });

    let response = server.put(&format!("/api/coffees/{}", created_coffee.id))
        .json(&update_payload)
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let updated_coffee: Coffee = response.json();
    assert_eq!(updated_coffee.name, "Partially Updated");
    assert_eq!(updated_coffee.price, 450);
    // Other fields should remain unchanged
    assert_eq!(updated_coffee.coffee_type, created_coffee.coffee_type);
    assert_eq!(updated_coffee.rating, created_coffee.rating);
}

/// Test updating a non-existent coffee
/// Requirements: 4.3, 8.3
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
    assert!(body["error"].as_str().unwrap().contains("not found"));
}

/// Test updating with invalid price (zero)
/// Requirements: 4.4, 8.4
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
        "price": 0
    });

    let response = server.put(&format!("/api/coffees/{}", created_coffee.id))
        .json(&update_payload)
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json();
    assert!(body["error"].as_str().unwrap().contains("Price must be a positive integer"));
}

/// Test updating with invalid price (negative)
/// Requirements: 4.4, 8.4
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
        "price": -50
    });

    let response = server.put(&format!("/api/coffees/{}", created_coffee.id))
        .json(&update_payload)
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json();
    assert!(body["error"].as_str().unwrap().contains("Price must be a positive integer"));
}

/// Test updating with invalid rating (below minimum)
/// Requirements: 4.4, 8.5
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
    assert!(body["error"].as_str().unwrap().contains("Rating must be between 0.0 and 5.0"));
}

/// Test updating with invalid rating (above maximum)
/// Requirements: 4.4, 8.5
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
    assert!(body["error"].as_str().unwrap().contains("Rating must be between 0.0 and 5.0"));
}

/// Test updating with invalid temperature
/// Requirements: 4.4, 8.6
#[tokio::test]
async fn test_update_coffee_invalid_temperature() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Create a coffee
    let payload = create_valid_coffee_payload("Test Coffee");
    let create_response = server.post("/api/coffees").json(&payload).await;
    let created_coffee: Coffee = create_response.json();

    // Try to update with invalid temperature
    let update_payload = json!({
        "temperature": "lukewarm"
    });

    let response = server.put(&format!("/api/coffees/{}", created_coffee.id))
        .json(&update_payload)
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json();
    assert!(body["error"].as_str().unwrap().contains("Temperature must be one of: hot, cold, both"));
}

// ============================================================================
// DELETE Coffee Tests (DELETE /api/coffees/:id)
// ============================================================================

/// Test deleting a coffee successfully
/// Requirements: 5.1, 5.2
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
/// Requirements: 5.3, 8.3
#[tokio::test]
async fn test_delete_coffee_not_found() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    let response = server.delete("/api/coffees/99999").await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    let body: serde_json::Value = response.json();
    assert!(body["error"].as_str().unwrap().contains("not found"));
}

/// Test deleting a coffee twice (idempotency check)
/// Requirements: 5.3
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
/// Requirements: 8.1, 9.4
#[tokio::test]
async fn test_error_response_format() {
    let pool = create_test_pool().await;
    let server = create_test_app(pool).await;

    // Trigger a 404 error
    let response = server.get("/api/coffees/99999").await;
    
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    let body: serde_json::Value = response.json();
    
    // Verify error field exists and is a string
    assert!(body.get("error").is_some());
    assert!(body["error"].is_string());
    assert!(!body["error"].as_str().unwrap().is_empty());
}
