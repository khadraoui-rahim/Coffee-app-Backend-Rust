use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Represents a coffee product in the database
/// 
/// Satisfies Requirements 1.1-1.10:
/// - 1.1: Unique identifier (id)
/// - 1.2: Name field as text
/// - 1.3: Coffee type field as text
/// - 1.4: Price field as integer (cents)
/// - 1.5: Rating field as decimal
/// - 1.6: Temperature field ("hot", "cold", or "both")
/// - 1.7: Description field as text
/// - 1.8: Size field as text
/// - 1.9: Liked field as boolean
/// - 1.10: Timestamp fields (created_at, updated_at)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Coffee {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = "Caffe Mocha")]
    pub name: String,
    #[schema(example = "Deep Foam")]
    pub coffee_type: String,
    /// Price in cents
    #[schema(example = 450)]
    pub price: i32,
    #[schema(example = 4.5, minimum = 0.0, maximum = 5.0)]
    pub rating: f64,
    #[schema(example = "hot", pattern = "hot|cold|both")]
    pub temperature: String, // "hot", "cold", or "both"
    #[schema(example = "Rich chocolate and espresso blend")]
    pub description: String,
    #[schema(example = "medium")]
    pub size: String,
    #[schema(example = false)]
    pub liked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Represents the data needed to create a new coffee product
/// 
/// Used for POST /api/coffees requests (Requirement 2.1)
/// All fields are required except id and timestamps which are auto-generated
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCoffee {
    #[schema(example = "Espresso")]
    pub name: String,
    #[schema(example = "Single Shot")]
    pub coffee_type: String,
    /// Price in cents
    #[schema(example = 350)]
    pub price: i32,
    #[schema(example = 4.5, minimum = 0.0, maximum = 5.0)]
    pub rating: f64,
    #[schema(example = "hot", pattern = "hot|cold|both")]
    pub temperature: String,
    #[schema(example = "Strong and bold")]
    pub description: String,
    #[schema(example = "small")]
    pub size: String,
    #[schema(example = true)]
    pub liked: bool,
}

/// Represents the data for updating an existing coffee product
/// 
/// Used for PUT /api/coffees/{id} requests (Requirement 4.1)
/// All fields are optional to support partial updates
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCoffee {
    #[schema(example = "Updated Name")]
    pub name: Option<String>,
    #[schema(example = "Updated Type")]
    pub coffee_type: Option<String>,
    /// Price in cents
    #[schema(example = 500)]
    pub price: Option<i32>,
    #[schema(example = 5.0, minimum = 0.0, maximum = 5.0)]
    pub rating: Option<f64>,
    #[schema(example = "cold", pattern = "hot|cold|both")]
    pub temperature: Option<String>,
    #[schema(example = "Updated description")]
    pub description: Option<String>,
    #[schema(example = "large")]
    pub size: Option<String>,
    #[schema(example = true)]
    pub liked: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    /// Test Coffee serialization to JSON
    /// Validates Requirements 1.1-1.10
    #[test]
    fn test_coffee_serialization() {
        let coffee = Coffee {
            id: 1,
            name: "Caffe Mocha".to_string(),
            coffee_type: "Deep Foam".to_string(),
            price: 453,
            rating: 4.8,
            temperature: "hot".to_string(),
            description: "Rich chocolate and espresso blend".to_string(),
            size: "medium".to_string(),
            liked: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&coffee).expect("Failed to serialize Coffee");
        
        // Verify JSON contains all required fields
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"name\":\"Caffe Mocha\""));
        assert!(json.contains("\"coffee_type\":\"Deep Foam\""));
        assert!(json.contains("\"price\":453"));
        assert!(json.contains("\"rating\":4.8"));
        assert!(json.contains("\"temperature\":\"hot\""));
        assert!(json.contains("\"description\":\"Rich chocolate and espresso blend\""));
        assert!(json.contains("\"size\":\"medium\""));
        assert!(json.contains("\"liked\":false"));
        assert!(json.contains("\"created_at\""));
        assert!(json.contains("\"updated_at\""));
    }

    /// Test CreateCoffee deserialization from JSON
    /// Validates Requirements 1.1-1.10 (excluding id and timestamps)
    #[test]
    fn test_create_coffee_deserialization() {
        let json = r#"{
            "name": "Espresso",
            "coffee_type": "Single Shot",
            "price": 250,
            "rating": 4.5,
            "temperature": "hot",
            "description": "Strong and bold",
            "size": "small",
            "liked": true
        }"#;

        let create_coffee: CreateCoffee = serde_json::from_str(json)
            .expect("Failed to deserialize CreateCoffee");

        assert_eq!(create_coffee.name, "Espresso");
        assert_eq!(create_coffee.coffee_type, "Single Shot");
        assert_eq!(create_coffee.price, 250);
        assert_eq!(create_coffee.rating, 4.5);
        assert_eq!(create_coffee.temperature, "hot");
        assert_eq!(create_coffee.description, "Strong and bold");
        assert_eq!(create_coffee.size, "small");
        assert_eq!(create_coffee.liked, true);
    }

    /// Test UpdateCoffee with all optional fields present
    /// Validates Requirements 4.1 (partial update support)
    #[test]
    fn test_update_coffee_all_fields() {
        let json = r#"{
            "name": "Updated Name",
            "coffee_type": "Updated Type",
            "price": 500,
            "rating": 5.0,
            "temperature": "cold",
            "description": "Updated description",
            "size": "large",
            "liked": true
        }"#;

        let update_coffee: UpdateCoffee = serde_json::from_str(json)
            .expect("Failed to deserialize UpdateCoffee");

        assert_eq!(update_coffee.name, Some("Updated Name".to_string()));
        assert_eq!(update_coffee.coffee_type, Some("Updated Type".to_string()));
        assert_eq!(update_coffee.price, Some(500));
        assert_eq!(update_coffee.rating, Some(5.0));
        assert_eq!(update_coffee.temperature, Some("cold".to_string()));
        assert_eq!(update_coffee.description, Some("Updated description".to_string()));
        assert_eq!(update_coffee.size, Some("large".to_string()));
        assert_eq!(update_coffee.liked, Some(true));
    }

    /// Test UpdateCoffee with partial fields (some fields omitted)
    /// Validates Requirements 4.1 (partial update support)
    #[test]
    fn test_update_coffee_partial_fields() {
        let json = r#"{
            "name": "Partial Update",
            "price": 350
        }"#;

        let update_coffee: UpdateCoffee = serde_json::from_str(json)
            .expect("Failed to deserialize UpdateCoffee");

        assert_eq!(update_coffee.name, Some("Partial Update".to_string()));
        assert_eq!(update_coffee.price, Some(350));
        assert_eq!(update_coffee.coffee_type, None);
        assert_eq!(update_coffee.rating, None);
        assert_eq!(update_coffee.temperature, None);
        assert_eq!(update_coffee.description, None);
        assert_eq!(update_coffee.size, None);
        assert_eq!(update_coffee.liked, None);
    }

    /// Test UpdateCoffee with no fields (empty update)
    /// Validates Requirements 4.1 (partial update support)
    #[test]
    fn test_update_coffee_empty() {
        let json = r#"{}"#;

        let update_coffee: UpdateCoffee = serde_json::from_str(json)
            .expect("Failed to deserialize UpdateCoffee");

        assert_eq!(update_coffee.name, None);
        assert_eq!(update_coffee.coffee_type, None);
        assert_eq!(update_coffee.price, None);
        assert_eq!(update_coffee.rating, None);
        assert_eq!(update_coffee.temperature, None);
        assert_eq!(update_coffee.description, None);
        assert_eq!(update_coffee.size, None);
        assert_eq!(update_coffee.liked, None);
    }
}
