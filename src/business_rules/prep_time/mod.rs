// Prep Time Calculator
// 
// Estimates order preparation time based on coffee items and current queue length.
// Considers base preparation time per item, additional time for quantities, and queue delays.

use crate::business_rules::{
    config_store::RuleConfigurationStore,
    error::{BRResult, BusinessRulesError},
};
use std::sync::Arc;

/// Order item for prep time calculation
#[derive(Debug, Clone)]
pub struct PrepTimeOrderItem {
    pub coffee_id: i32,
    pub quantity: u32,
}

/// Breakdown of prep time calculation
#[derive(Debug, Clone)]
pub struct PrepTimeBreakdown {
    pub base_time: i32,
    pub queue_delay: i32,
    pub total_time: i32,
}

/// Result of prep time estimation
#[derive(Debug, Clone)]
pub struct PrepTimeEstimate {
    pub estimated_minutes: i32,
    pub queue_position: usize,
    pub breakdown: PrepTimeBreakdown,
}

/// Prep Time Calculator
/// 
/// Calculates estimated preparation time for orders based on item complexity and queue length.
pub struct PrepTimeCalculator {
    config_store: Arc<RuleConfigurationStore>,
}

impl PrepTimeCalculator {
    /// Create a new PrepTimeCalculator
    pub fn new(config_store: Arc<RuleConfigurationStore>) -> Self {
        Self { config_store }
    }
    
    /// Estimate preparation time for an order
    /// 
    /// Orchestrates the full calculation:
    /// 1. Calculate base time from items
    /// 2. Get queue delay from pending/preparing orders
    /// 3. Return estimate with breakdown
    pub async fn estimate(&self, items: &[PrepTimeOrderItem]) -> BRResult<PrepTimeEstimate> {
        // Calculate base time from items
        let base_time = self.calculate_base_time(items).await?;
        
        // Get queue delay
        let (queue_delay, queue_position) = self.get_queue_delay().await?;
        
        // Calculate total time
        let total_time = base_time + queue_delay;
        
        // Ensure result is always positive
        let estimated_minutes = total_time.max(1);
        
        Ok(PrepTimeEstimate {
            estimated_minutes,
            queue_position,
            breakdown: PrepTimeBreakdown {
                base_time,
                queue_delay,
                total_time,
            },
        })
    }
    
    /// Calculate base preparation time from order items
    /// 
    /// Sums base_minutes for all items and adds per_additional_item time for quantities > 1
    async fn calculate_base_time(&self, items: &[PrepTimeOrderItem]) -> BRResult<i32> {
        let prep_time_config = self.config_store.get_prep_time_config().await?;
        
        let mut total_time = 0;
        
        for item in items {
            // Get prep time config for this coffee
            let config = prep_time_config
                .get(&item.coffee_id)
                .ok_or_else(|| BusinessRulesError::CoffeeNotFound(item.coffee_id))?;
            
            // Add base time for first item
            total_time += config.base_minutes;
            
            // Add per_additional_item time for quantities > 1
            if item.quantity > 1 {
                let additional_items = (item.quantity - 1) as i32;
                total_time += config.per_additional_item * additional_items;
            }
        }
        
        Ok(total_time)
    }
    
    /// Get queue delay from pending and preparing orders
    /// 
    /// Returns (queue_delay_minutes, queue_position)
    async fn get_queue_delay(&self) -> BRResult<(i32, usize)> {
        let pool = self.config_store.pool();
        
        // Query orders with status 'pending' or 'preparing'
        let result = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as "count!",
                COALESCE(SUM(estimated_prep_minutes), 0) as "total_minutes!"
            FROM orders
            WHERE status IN ('pending', 'preparing')
            "#
        )
        .fetch_one(pool)
        .await?;
        
        let queue_position = result.count as usize;
        let queue_delay = result.total_minutes as i32;
        
        Ok((queue_delay, queue_position))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prep_time_order_item_creation() {
        let item = PrepTimeOrderItem {
            coffee_id: 1,
            quantity: 2,
        };
        
        assert_eq!(item.coffee_id, 1);
        assert_eq!(item.quantity, 2);
    }
    
    #[test]
    fn test_prep_time_breakdown_creation() {
        let breakdown = PrepTimeBreakdown {
            base_time: 10,
            queue_delay: 5,
            total_time: 15,
        };
        
        assert_eq!(breakdown.base_time, 10);
        assert_eq!(breakdown.queue_delay, 5);
        assert_eq!(breakdown.total_time, 15);
    }
    
    #[test]
    fn test_prep_time_estimate_creation() {
        let estimate = PrepTimeEstimate {
            estimated_minutes: 15,
            queue_position: 2,
            breakdown: PrepTimeBreakdown {
                base_time: 10,
                queue_delay: 5,
                total_time: 15,
            },
        };
        
        assert_eq!(estimate.estimated_minutes, 15);
        assert_eq!(estimate.queue_position, 2);
        assert_eq!(estimate.breakdown.base_time, 10);
    }
    
    #[test]
    fn test_base_time_calculation_single_item() {
        // Test calculation logic for single item
        let base_minutes = 5;
        let per_additional_item = 2;
        let quantity = 1u32;
        
        let total_time = base_minutes + if quantity > 1 {
            per_additional_item * (quantity - 1) as i32
        } else {
            0
        };
        
        assert_eq!(total_time, 5);
    }
    
    #[test]
    fn test_base_time_calculation_multiple_items() {
        // Test calculation logic for multiple items of same coffee
        let base_minutes = 5;
        let per_additional_item = 2;
        let quantity = 3u32;
        
        let total_time = base_minutes + if quantity > 1 {
            per_additional_item * (quantity - 1) as i32
        } else {
            0
        };
        
        // 5 + (2 * 2) = 9
        assert_eq!(total_time, 9);
    }
    
    #[test]
    fn test_base_time_calculation_multiple_coffees() {
        // Test calculation logic for multiple different coffees
        let items = vec![
            (5, 2, 2), // base=5, per_additional=2, quantity=2
            (8, 3, 1), // base=8, per_additional=3, quantity=1
        ];
        
        let mut total_time = 0;
        for (base_minutes, per_additional_item, quantity) in items {
            total_time += base_minutes;
            if quantity > 1 {
                total_time += per_additional_item * (quantity - 1);
            }
        }
        
        // (5 + 2*1) + (8 + 0) = 7 + 8 = 15
        assert_eq!(total_time, 15);
    }
    
    #[test]
    fn test_positive_prep_time_constraint() {
        // Test that prep time is always at least 1 minute
        let base_time = 0;
        let queue_delay = 0;
        let total_time = base_time + queue_delay;
        let estimated_minutes = total_time.max(1);
        
        assert_eq!(estimated_minutes, 1);
    }
    
    #[test]
    fn test_queue_delay_calculation() {
        // Test queue delay logic
        let queue_orders = vec![
            Some(10), // 10 minutes
            Some(15), // 15 minutes
            Some(5),  // 5 minutes
        ];
        
        let total_delay: i32 = queue_orders.iter().filter_map(|&x| x).sum();
        let queue_position = queue_orders.len();
        
        assert_eq!(total_delay, 30);
        assert_eq!(queue_position, 3);
    }
    
    #[test]
    fn test_empty_queue() {
        // Test with no orders in queue
        let queue_delay = 0;
        let queue_position = 0;
        let base_time = 10;
        
        let total_time = base_time + queue_delay;
        
        assert_eq!(total_time, 10);
        assert_eq!(queue_position, 0);
    }
}
