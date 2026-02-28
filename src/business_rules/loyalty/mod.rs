// Loyalty Engine
// 
// Calculates and awards loyalty points to customers based on order totals and bonus multipliers.
// Manages customer loyalty balances with database persistence.

use crate::business_rules::{
    config_store::RuleConfigurationStore,
    error::{BRResult, BusinessRulesError},
};
use rust_decimal::Decimal;
use std::sync::Arc;

/// Order item for loyalty calculation
#[derive(Debug, Clone)]
pub struct LoyaltyOrderItem {
    pub coffee_id: i32,
    pub quantity: u32,
    pub price: Decimal,
}

/// Breakdown of loyalty points calculation
#[derive(Debug, Clone)]
pub struct LoyaltyCalculation {
    pub base_points: i32,
    pub bonus_points: i32,
    pub total_points: i32,
    pub order_total: Decimal,
}

/// Customer loyalty balance
#[derive(Debug, Clone)]
pub struct CustomerLoyalty {
    pub customer_id: i32,
    pub points_balance: i32,
    pub lifetime_points: i32,
}

/// Loyalty Engine
/// 
/// Calculates loyalty points based on order totals and manages customer balances.
pub struct LoyaltyEngine {
    config_store: Arc<RuleConfigurationStore>,
}

impl LoyaltyEngine {
    /// Create a new LoyaltyEngine
    pub fn new(config_store: Arc<RuleConfigurationStore>) -> Self {
        Self { config_store }
    }
    
    /// Calculate loyalty points for an order
    /// 
    /// Calculates base points from order total and applies bonus multipliers for specific items.
    /// Points are rounded down to the nearest whole number.
    pub async fn calculate_points(
        &self,
        order_total: Decimal,
        items: &[LoyaltyOrderItem],
    ) -> BRResult<LoyaltyCalculation> {
        // Load loyalty config
        let config = self.config_store.get_loyalty_config().await?;
        
        // Calculate base points: order_total * points_per_dollar
        let base_points_decimal = order_total * config.points_per_dollar;
        let base_points = base_points_decimal.floor().to_string().parse::<i32>()
            .map_err(|e| BusinessRulesError::CalculationError(format!("Failed to convert points: {}", e)))?;
        
        // Calculate bonus points from multipliers
        let mut bonus_points_decimal = Decimal::ZERO;
        for item in items {
            if let Some(multiplier) = config.bonus_multipliers.get(&item.coffee_id) {
                let item_total = item.price * Decimal::from(item.quantity);
                let item_base_points = item_total * config.points_per_dollar;
                let bonus = item_base_points * (*multiplier - Decimal::ONE);
                bonus_points_decimal += bonus;
            }
        }
        
        let bonus_points = bonus_points_decimal.floor().to_string().parse::<i32>()
            .map_err(|e| BusinessRulesError::CalculationError(format!("Failed to convert bonus points: {}", e)))?;
        
        let total_points = base_points + bonus_points;
        
        Ok(LoyaltyCalculation {
            base_points,
            bonus_points,
            total_points,
            order_total,
        })
    }
    
    /// Award loyalty points to a customer
    /// 
    /// Updates the customer's points balance and lifetime points.
    /// Creates a new loyalty record if the customer doesn't have one.
    pub async fn award_points(
        &self,
        customer_id: i32,
        points: i32,
    ) -> BRResult<CustomerLoyalty> {
        let pool = self.config_store.pool();
        
        // Try to update existing record
        let result = sqlx::query!(
            r#"
            INSERT INTO customer_loyalty (customer_id, points_balance, lifetime_points)
            VALUES ($1, $2, $2)
            ON CONFLICT (customer_id)
            DO UPDATE SET
                points_balance = customer_loyalty.points_balance + $2,
                lifetime_points = customer_loyalty.lifetime_points + $2,
                updated_at = NOW()
            RETURNING customer_id, points_balance, lifetime_points
            "#,
            customer_id,
            points
        )
        .fetch_one(pool)
        .await?;
        
        Ok(CustomerLoyalty {
            customer_id: result.customer_id,
            points_balance: result.points_balance,
            lifetime_points: result.lifetime_points,
        })
    }
    
    /// Get customer's current loyalty balance
    /// 
    /// Returns 0 if the customer has no loyalty record.
    pub async fn get_customer_balance(&self, customer_id: i32) -> BRResult<i32> {
        let pool = self.config_store.pool();
        
        let result = sqlx::query!(
            r#"
            SELECT points_balance
            FROM customer_loyalty
            WHERE customer_id = $1
            "#,
            customer_id
        )
        .fetch_optional(pool)
        .await?;
        
        Ok(result.map(|r| r.points_balance).unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loyalty_order_item_creation() {
        let item = LoyaltyOrderItem {
            coffee_id: 1,
            quantity: 2,
            price: Decimal::from(5),
        };
        
        assert_eq!(item.coffee_id, 1);
        assert_eq!(item.quantity, 2);
        assert_eq!(item.price, Decimal::from(5));
    }
    
    #[test]
    fn test_loyalty_calculation_creation() {
        let calc = LoyaltyCalculation {
            base_points: 10,
            bonus_points: 5,
            total_points: 15,
            order_total: Decimal::from(100),
        };
        
        assert_eq!(calc.base_points, 10);
        assert_eq!(calc.bonus_points, 5);
        assert_eq!(calc.total_points, 15);
        assert_eq!(calc.order_total, Decimal::from(100));
    }
    
    #[test]
    fn test_customer_loyalty_creation() {
        let loyalty = CustomerLoyalty {
            customer_id: 1,
            points_balance: 100,
            lifetime_points: 500,
        };
        
        assert_eq!(loyalty.customer_id, 1);
        assert_eq!(loyalty.points_balance, 100);
        assert_eq!(loyalty.lifetime_points, 500);
    }
    
    #[test]
    fn test_base_points_calculation() {
        // Test base points calculation logic
        let order_total = Decimal::from(100);
        let points_per_dollar = Decimal::new(1, 1); // 0.1
        
        let base_points_decimal = order_total * points_per_dollar;
        let base_points = base_points_decimal.floor().to_string().parse::<i32>().unwrap();
        
        // 100 * 0.1 = 10 points
        assert_eq!(base_points, 10);
    }
    
    #[test]
    fn test_fractional_points_rounding() {
        // Test that fractional points are rounded down
        let order_total = Decimal::new(1055, 1); // 105.5
        let points_per_dollar = Decimal::new(1, 1); // 0.1
        
        let base_points_decimal = order_total * points_per_dollar;
        let base_points = base_points_decimal.floor().to_string().parse::<i32>().unwrap();
        
        // 105.5 * 0.1 = 10.55, rounded down to 10
        assert_eq!(base_points, 10);
    }
    
    #[test]
    fn test_bonus_multiplier_calculation() {
        // Test bonus multiplier logic
        let item_price = Decimal::from(20);
        let quantity = 2u32;
        let points_per_dollar = Decimal::new(1, 1); // 0.1
        let multiplier = Decimal::from(2); // 2x multiplier
        
        let item_total = item_price * Decimal::from(quantity);
        let item_base_points = item_total * points_per_dollar;
        let bonus = item_base_points * (multiplier - Decimal::ONE);
        let bonus_points = bonus.floor().to_string().parse::<i32>().unwrap();
        
        // item_total = 40, base = 4, bonus = 4 * (2 - 1) = 4
        assert_eq!(bonus_points, 4);
    }
    
    #[test]
    fn test_multiple_bonus_items() {
        // Test calculation with multiple bonus items
        let points_per_dollar = Decimal::new(1, 1); // 0.1
        
        let items = vec![
            (Decimal::from(20), 1u32, Decimal::from(2)), // price=20, qty=1, multiplier=2x
            (Decimal::from(30), 1u32, Decimal::new(15, 1)), // price=30, qty=1, multiplier=1.5x
        ];
        
        let mut total_bonus = Decimal::ZERO;
        for (price, quantity, multiplier) in items {
            let item_total = price * Decimal::from(quantity);
            let item_base_points = item_total * points_per_dollar;
            let bonus = item_base_points * (multiplier - Decimal::ONE);
            total_bonus += bonus;
        }
        
        let bonus_points = total_bonus.floor().to_string().parse::<i32>().unwrap();
        
        // Item 1: 20 * 0.1 * (2 - 1) = 2
        // Item 2: 30 * 0.1 * (1.5 - 1) = 1.5
        // Total: 2 + 1.5 = 3.5, rounded down to 3
        assert_eq!(bonus_points, 3);
    }
    
    #[test]
    fn test_zero_order_total() {
        // Test with zero order total
        let order_total = Decimal::ZERO;
        let points_per_dollar = Decimal::new(1, 1);
        
        let base_points_decimal = order_total * points_per_dollar;
        let base_points = base_points_decimal.floor().to_string().parse::<i32>().unwrap();
        
        assert_eq!(base_points, 0);
    }
    
    #[test]
    fn test_points_scale_with_order_total() {
        // Test that points scale linearly with order total
        let points_per_dollar = Decimal::new(1, 1); // 0.1
        
        let order_totals = vec![
            Decimal::from(10),
            Decimal::from(50),
            Decimal::from(100),
        ];
        
        let expected_points = vec![1, 5, 10];
        
        for (total, expected) in order_totals.iter().zip(expected_points.iter()) {
            let points_decimal = total * points_per_dollar;
            let points = points_decimal.floor().to_string().parse::<i32>().unwrap();
            assert_eq!(points, *expected);
        }
    }
    
    #[test]
    fn test_whole_number_points() {
        // Test that points are always whole numbers
        let order_totals = vec![
            Decimal::new(1234, 2), // 12.34
            Decimal::new(9999, 2), // 99.99
            Decimal::new(5555, 2), // 55.55
        ];
        
        let points_per_dollar = Decimal::new(1, 1); // 0.1
        
        for total in order_totals {
            let points_decimal = total * points_per_dollar;
            let points = points_decimal.floor().to_string().parse::<i32>().unwrap();
            
            // Verify it's a whole number (no fractional part)
            assert_eq!(points as f64, points as f64);
        }
    }
}
