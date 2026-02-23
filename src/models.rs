use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use validator::Validate;

/// Represents a coffee product in the database
/// Matches the frontend CoffeeItem model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Coffee {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = "https://images.unsplash.com/photo-1594146971821-373461fd5cd8")]
    pub image_url: String,
    #[schema(example = "Caffe Mocha")]
    pub name: String,
    #[schema(example = "Deep Foam")]
    pub coffee_type: String,
    #[schema(example = 4.53)]
    pub price: f64,
    #[schema(example = 4.8, minimum = 0.0, maximum = 5.0)]
    pub rating: f64,
}

/// Represents the data needed to create a new coffee product
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateCoffee {
    #[schema(example = "https://images.unsplash.com/photo-1594146971821-373461fd5cd8")]
    #[validate(length(min = 1, message = "Image URL is required"))]
    #[validate(url(message = "Image URL must be a valid URL"))]
    pub image_url: String,
    
    #[schema(example = "Espresso")]
    #[validate(length(min = 1, max = 100, message = "Name must be between 1 and 100 characters"))]
    pub name: String,
    
    #[schema(example = "Single Shot")]
    #[validate(length(min = 1, max = 100, message = "Coffee type must be between 1 and 100 characters"))]
    pub coffee_type: String,
    
    #[schema(example = 3.50)]
    #[validate(custom = "crate::validation::validate_positive_price")]
    pub price: f64,
    
    #[schema(example = 4.5, minimum = 0.0, maximum = 5.0)]
    #[validate(custom = "crate::validation::validate_rating_range")]
    pub rating: f64,
}

/// Represents the data for updating an existing coffee product
/// All fields are optional to support partial updates
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateCoffee {
    #[schema(example = "https://images.unsplash.com/photo-1594146971821-373461fd5cd8")]
    #[validate(url(message = "Image URL must be a valid URL"))]
    pub image_url: Option<String>,
    
    #[schema(example = "Updated Name")]
    #[validate(length(min = 1, max = 100, message = "Name must be between 1 and 100 characters"))]
    pub name: Option<String>,
    
    #[schema(example = "Updated Type")]
    #[validate(length(min = 1, max = 100, message = "Coffee type must be between 1 and 100 characters"))]
    pub coffee_type: Option<String>,
    
    #[schema(example = 5.00)]
    #[validate(custom(function = "crate::validation::validate_optional_positive_price"))]
    pub price: Option<f64>,
    
    #[schema(example = 5.0, minimum = 0.0, maximum = 5.0)]
    #[validate(custom(function = "crate::validation::validate_optional_rating_range"))]
    pub rating: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test Coffee serialization to JSON
    #[test]
    fn test_coffee_serialization() {
        let coffee = Coffee {
            id: 1,
            image_url: "https://images.unsplash.com/photo-1594146971821-373461fd5cd8".to_string(),
            name: "Caffe Mocha".to_string(),
            coffee_type: "Deep Foam".to_string(),
            price: 4.53,
            rating: 4.8,
        };

        let json = serde_json::to_string(&coffee).expect("Failed to serialize Coffee");
        
        // Verify JSON contains all required fields
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"image_url\""));
        assert!(json.contains("\"name\":\"Caffe Mocha\""));
        assert!(json.contains("\"coffee_type\":\"Deep Foam\""));
        assert!(json.contains("\"price\":4.53"));
        assert!(json.contains("\"rating\":4.8"));
    }

    /// Test CreateCoffee deserialization from JSON
    #[test]
    fn test_create_coffee_deserialization() {
        let json = r#"{
            "image_url": "https://images.unsplash.com/photo-1594146971821-373461fd5cd8",
            "name": "Espresso",
            "coffee_type": "Single Shot",
            "price": 3.50,
            "rating": 4.5
        }"#;

        let create_coffee: CreateCoffee = serde_json::from_str(json)
            .expect("Failed to deserialize CreateCoffee");

        assert_eq!(create_coffee.image_url, "https://images.unsplash.com/photo-1594146971821-373461fd5cd8");
        assert_eq!(create_coffee.name, "Espresso");
        assert_eq!(create_coffee.coffee_type, "Single Shot");
        assert_eq!(create_coffee.price, 3.50);
        assert_eq!(create_coffee.rating, 4.5);
    }

    /// Test UpdateCoffee with all optional fields present
    #[test]
    fn test_update_coffee_all_fields() {
        let json = r#"{
            "image_url": "https://images.unsplash.com/photo-new",
            "name": "Updated Name",
            "coffee_type": "Updated Type",
            "price": 5.00,
            "rating": 5.0
        }"#;

        let update_coffee: UpdateCoffee = serde_json::from_str(json)
            .expect("Failed to deserialize UpdateCoffee");

        assert_eq!(update_coffee.image_url, Some("https://images.unsplash.com/photo-new".to_string()));
        assert_eq!(update_coffee.name, Some("Updated Name".to_string()));
        assert_eq!(update_coffee.coffee_type, Some("Updated Type".to_string()));
        assert_eq!(update_coffee.price, Some(5.00));
        assert_eq!(update_coffee.rating, Some(5.0));
    }

    /// Test UpdateCoffee with partial fields (some fields omitted)
    #[test]
    fn test_update_coffee_partial_fields() {
        let json = r#"{
            "name": "Partial Update",
            "price": 3.50
        }"#;

        let update_coffee: UpdateCoffee = serde_json::from_str(json)
            .expect("Failed to deserialize UpdateCoffee");

        assert_eq!(update_coffee.name, Some("Partial Update".to_string()));
        assert_eq!(update_coffee.price, Some(3.50));
        assert_eq!(update_coffee.image_url, None);
        assert_eq!(update_coffee.coffee_type, None);
        assert_eq!(update_coffee.rating, None);
    }

    /// Test UpdateCoffee with no fields (empty update)
    #[test]
    fn test_update_coffee_empty() {
        let json = r#"{}"#;

        let update_coffee: UpdateCoffee = serde_json::from_str(json)
            .expect("Failed to deserialize UpdateCoffee");

        assert_eq!(update_coffee.image_url, None);
        assert_eq!(update_coffee.name, None);
        assert_eq!(update_coffee.coffee_type, None);
        assert_eq!(update_coffee.price, None);
        assert_eq!(update_coffee.rating, None);
    }
}
