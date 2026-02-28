// Availability Engine
// 
// Manages coffee availability rules and validates orders against availability constraints.
// Ensures customers can only order items that are currently available.

use crate::business_rules::{
    config_store::{CoffeeAvailability, RuleConfigurationStore},
    error::{BRResult, BusinessRulesError},
    types::AvailabilityStatus,
};
use chrono::Utc;
use std::sync::Arc;

/// Order item for validation
/// 
/// Represents a single item in an order that needs availability checking.
#[derive(Debug, Clone)]
pub struct OrderItem {
    pub coffee_id: i32,
    pub quantity: u32,
}

/// Validation error for a specific order item
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub coffee_id: i32,
    pub coffee_name: Option<String>,
    pub reason: String,
}

/// Result of order validation
#[derive(Debug, Clone)]
pub struct OrderValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

/// Availability Engine
/// 
/// Checks coffee availability and validates orders against availability rules.
pub struct AvailabilityEngine {
    config_store: Arc<RuleConfigurationStore>,
}

impl AvailabilityEngine {
    /// Create a new AvailabilityEngine
    pub fn new(config_store: Arc<RuleConfigurationStore>) -> Self {
        Self { config_store }
    }
    
    /// Check availability for a single coffee item
    /// 
    /// Returns the availability status and reason if unavailable.
    pub async fn check_coffee_availability(&self, coffee_id: i32) -> BRResult<CoffeeAvailability> {
        let availability_rules = self.config_store.get_availability_rules().await?;
        
        // If no rule exists for this coffee, assume it's available
        let availability = availability_rules.get(&coffee_id).cloned().unwrap_or_else(|| {
            CoffeeAvailability {
                coffee_id,
                status: AvailabilityStatus::Available,
                reason: None,
                available_from: None,
                available_until: None,
                updated_at: Utc::now(),
            }
        });
        
        // Check time-based availability if specified
        if let Some(available_from) = availability.available_from {
            if Utc::now() < available_from {
                return Ok(CoffeeAvailability {
                    status: AvailabilityStatus::Seasonal,
                    reason: Some(format!("Not available until {}", available_from.format("%Y-%m-%d %H:%M"))),
                    ..availability
                });
            }
        }
        
        if let Some(available_until) = availability.available_until {
            if Utc::now() > available_until {
                return Ok(CoffeeAvailability {
                    status: AvailabilityStatus::Seasonal,
                    reason: Some(format!("No longer available after {}", available_until.format("%Y-%m-%d %H:%M"))),
                    ..availability
                });
            }
        }
        
        Ok(availability)
    }
    
    /// Validate all items in an order
    /// 
    /// Checks each item's availability and collects all errors.
    /// Returns a validation result with all unavailable items listed.
    pub async fn validate_order_items(&self, items: &[OrderItem]) -> BRResult<OrderValidationResult> {
        let mut errors = Vec::new();
        let warnings = Vec::new();
        
        for item in items {
            match self.check_coffee_availability(item.coffee_id).await {
                Ok(availability) => {
                    match availability.status {
                        AvailabilityStatus::Available => {
                            // Item is available, no action needed
                        }
                        AvailabilityStatus::OutOfStock => {
                            errors.push(ValidationError {
                                coffee_id: item.coffee_id,
                                coffee_name: None,
                                reason: availability.reason.unwrap_or_else(|| "Out of stock".to_string()),
                            });
                        }
                        AvailabilityStatus::Seasonal => {
                            errors.push(ValidationError {
                                coffee_id: item.coffee_id,
                                coffee_name: None,
                                reason: availability.reason.unwrap_or_else(|| "Seasonal item not currently available".to_string()),
                            });
                        }
                        AvailabilityStatus::Discontinued => {
                            errors.push(ValidationError {
                                coffee_id: item.coffee_id,
                                coffee_name: None,
                                reason: availability.reason.unwrap_or_else(|| "Item has been discontinued".to_string()),
                            });
                        }
                    }
                }
                Err(e) => {
                    // If we can't check availability, treat as an error
                    errors.push(ValidationError {
                        coffee_id: item.coffee_id,
                        coffee_name: None,
                        reason: format!("Unable to verify availability: {}", e),
                    });
                }
            }
        }
        
        Ok(OrderValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        })
    }
    
    /// Update availability status for a coffee item
    /// 
    /// Updates the database and invalidates the cache.
    pub async fn update_availability(
        &self,
        coffee_id: i32,
        status: AvailabilityStatus,
        reason: Option<String>,
    ) -> BRResult<()> {
        // Update in database
        sqlx::query!(
            r#"
            INSERT INTO coffee_availability (coffee_id, status, reason, updated_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (coffee_id)
            DO UPDATE SET
                status = $2,
                reason = $3,
                updated_at = NOW()
            "#,
            coffee_id,
            status.to_string(),
            reason
        )
        .execute(self.config_store.pool())
        .await?;
        
        // Invalidate cache to force reload
        self.config_store.invalidate_cache("availability").await;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::business_rules::types::AvailabilityStatus;

    #[test]
    fn test_order_item_creation() {
        let item = OrderItem {
            coffee_id: 1,
            quantity: 2,
        };
        
        assert_eq!(item.coffee_id, 1);
        assert_eq!(item.quantity, 2);
    }
    
    #[test]
    fn test_validation_error_creation() {
        let error = ValidationError {
            coffee_id: 1,
            coffee_name: Some("Espresso".to_string()),
            reason: "Out of stock".to_string(),
        };
        
        assert_eq!(error.coffee_id, 1);
        assert_eq!(error.coffee_name, Some("Espresso".to_string()));
        assert_eq!(error.reason, "Out of stock");
    }
    
    #[test]
    fn test_order_validation_result_valid() {
        let result = OrderValidationResult {
            is_valid: true,
            errors: vec![],
            warnings: vec![],
        };
        
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }
    
    #[test]
    fn test_order_validation_result_invalid() {
        let result = OrderValidationResult {
            is_valid: false,
            errors: vec![
                ValidationError {
                    coffee_id: 1,
                    coffee_name: Some("Espresso".to_string()),
                    reason: "Out of stock".to_string(),
                },
            ],
            warnings: vec![],
        };
        
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].coffee_id, 1);
    }
    
    #[test]
    fn test_availability_status_matching() {
        let statuses = vec![
            AvailabilityStatus::Available,
            AvailabilityStatus::OutOfStock,
            AvailabilityStatus::Seasonal,
            AvailabilityStatus::Discontinued,
        ];
        
        for status in statuses {
            match status {
                AvailabilityStatus::Available => assert!(true),
                AvailabilityStatus::OutOfStock => assert!(true),
                AvailabilityStatus::Seasonal => assert!(true),
                AvailabilityStatus::Discontinued => assert!(true),
            }
        }
    }
}
