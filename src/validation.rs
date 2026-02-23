// Validation utilities module
// Provides custom validation functions for domain-specific rules

use validator::ValidationError;

/// Validates that a roast level is one of the accepted values
/// Valid values: "light", "medium", "dark" (case-insensitive)
pub fn validate_roast_level(roast: &str) -> Result<(), ValidationError> {
    let valid_levels = ["light", "medium", "dark"];
    if valid_levels.contains(&roast.to_lowercase().as_str()) {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_roast_level"))
    }
}

/// Validates that price is positive (for required f64 fields)
pub fn validate_positive_price(price: f64) -> Result<(), ValidationError> {
    if price <= 0.0 {
        Err(ValidationError::new("price_must_be_positive"))
    } else {
        Ok(())
    }
}

/// Validates that rating is between 0.0 and 5.0 (for required f64 fields)
pub fn validate_rating_range(rating: f64) -> Result<(), ValidationError> {
    if rating < 0.0 || rating > 5.0 {
        Err(ValidationError::new("rating_out_of_range"))
    } else {
        Ok(())
    }
}

/// Validates that optional price is positive (for Option<f64> fields)
pub fn validate_optional_positive_price(price: f64) -> Result<(), ValidationError> {
    validate_positive_price(price)
}

/// Validates that optional rating is between 0.0 and 5.0 (for Option<f64> fields)
pub fn validate_optional_rating_range(rating: f64) -> Result<(), ValidationError> {
    validate_rating_range(rating)
}
