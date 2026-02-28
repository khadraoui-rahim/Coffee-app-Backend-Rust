// Business Rules System Module
// 
// This module provides a flexible, data-driven business rules engine for the coffee shop backend.
// It manages four core capabilities:
// - Availability management: Control which coffees can be ordered
// - Dynamic pricing: Apply configurable pricing rules and discounts
// - Preparation time estimation: Calculate order prep time based on items and queue
// - Loyalty points: Calculate and award customer loyalty points
//
// The system is designed to be configurable through database settings without code deployments.

pub mod error;
pub mod types;
pub mod config_store;
pub mod availability;
pub mod pricing;
pub mod prep_time;
pub mod loyalty;
pub mod audit;
pub mod handlers;
pub mod metrics;

// Re-export commonly used types for convenience
pub use error::{BusinessRulesError, BRResult};
pub use types::{
    AvailabilityStatus,
    DiscountType,
    CombinationStrategy,
    PricingRuleType,
};
pub use config_store::{
    RuleConfigurationStore,
    CoffeeAvailability,
    PricingRule,
    TimeBasedRuleConfig,
    TimeRange,
    QuantityBasedRuleConfig,
    PromotionalRuleConfig,
    CoffeeBaseTime,
    LoyaltyConfig,
};
pub use availability::{
    AvailabilityEngine,
    OrderItem,
    ValidationError,
    OrderValidationResult,
};
pub use pricing::{
    PricingEngine,
    PricingOrderItem,
    AppliedPricingRule,
    OrderPricingResult,
};
pub use prep_time::{
    PrepTimeCalculator,
    PrepTimeOrderItem,
    PrepTimeEstimate,
    PrepTimeBreakdown,
};
pub use loyalty::{
    LoyaltyEngine,
    LoyaltyOrderItem,
    LoyaltyCalculation,
    CustomerLoyalty,
};
pub use audit::{
    AuditLogger,
    AuditRecord,
};
pub use metrics::PerformanceMetrics;

// Business Rules Engine - Orchestrator
// 
// Coordinates all business rules engines and provides a unified interface.

use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use serde_json::json;

/// Business Rules Engine
/// 
/// Orchestrates all business rules engines (availability, pricing, prep time, loyalty)
/// and provides a unified interface for applying business rules to orders.
pub struct BusinessRulesEngine {
    availability_engine: AvailabilityEngine,
    pricing_engine: PricingEngine,
    prep_time_calculator: PrepTimeCalculator,
    loyalty_engine: LoyaltyEngine,
    audit_logger: AuditLogger,
    metrics: Arc<PerformanceMetrics>,
    config_store: Arc<RuleConfigurationStore>,
}

impl BusinessRulesEngine {
    /// Create a new BusinessRulesEngine
    /// 
    /// Initializes all sub-engines with a shared configuration store and audit logger.
    pub fn new(pool: PgPool) -> Self {
        let metrics = Arc::new(PerformanceMetrics::new());
        let config_store = Arc::new(RuleConfigurationStore::with_metrics(
            pool.clone(),
            metrics.clone(),
        ));
        let audit_logger = AuditLogger::new(pool);
        
        Self {
            availability_engine: AvailabilityEngine::new(config_store.clone()),
            pricing_engine: PricingEngine::new(config_store.clone()),
            prep_time_calculator: PrepTimeCalculator::new(config_store.clone()),
            loyalty_engine: LoyaltyEngine::new(config_store.clone()),
            audit_logger,
            metrics: metrics.clone(),
            config_store,
        }
    }
    
    /// Get performance metrics
    pub fn metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }
    
    /// Warm up the cache by loading all configurations
    /// 
    /// Should be called on application startup to pre-load configurations
    /// and avoid cold-start latency on first requests.
    pub async fn warm_cache(&self) -> BRResult<()> {
        tracing::info!("Warming business rules cache...");
        
        // Load all configuration types
        let _ = self.config_store.get_availability_rules().await?;
        let _ = self.config_store.get_pricing_rules().await?;
        let _ = self.config_store.get_prep_time_config().await?;
        let _ = self.config_store.get_loyalty_config().await?;
        
        tracing::info!("Business rules cache warmed successfully");
        Ok(())
    }
    
    /// Validate order items for availability
    /// 
    /// Checks if all items in the order are available and logs the validation result.
    pub async fn validate_order(
        &self,
        order_id: Uuid,
        items: &[OrderItem],
    ) -> BRResult<OrderValidationResult> {
        let _timer = self.metrics.start_availability_check();
        
        // Validate items
        let result = self.availability_engine.validate_order_items(items).await?;
        
        // Log validation result
        let rule_data = json!({
            "items_checked": items.len(),
            "is_valid": result.is_valid,
            "errors_count": result.errors.len(),
        });
        
        let effect = if result.is_valid {
            "All items available".to_string()
        } else {
            format!("{} items unavailable", result.errors.len())
        };
        
        self.audit_logger.log_availability_check(order_id, rule_data, &effect).await;
        
        Ok(result)
    }
    
    /// Calculate order price with applicable rules
    /// 
    /// Applies pricing rules and logs the calculation result.
    pub async fn calculate_price(
        &self,
        order_id: Uuid,
        items: &[PricingOrderItem],
        strategy: CombinationStrategy,
    ) -> BRResult<OrderPricingResult> {
        let _timer = self.metrics.start_pricing_calculation();
        
        // Calculate price
        let result = self.pricing_engine.calculate_order_price(items, strategy).await?;
        
        // Log pricing application
        let rule_data = json!({
            "base_price": result.base_price,
            "final_price": result.final_price,
            "total_discount": result.total_discount,
            "rules_applied": result.applied_rules.len(),
            "strategy": format!("{:?}", strategy),
        });
        
        let effect = format!(
            "Applied {} rules, discount: ${:.2}",
            result.applied_rules.len(),
            result.total_discount
        );
        
        // Log each applied rule
        for applied_rule in &result.applied_rules {
            self.audit_logger.log_pricing_application(
                order_id,
                Some(applied_rule.rule_id),
                json!({
                    "rule_type": format!("{:?}", applied_rule.rule_type),
                    "description": &applied_rule.description,
                    "discount_amount": applied_rule.discount_amount,
                }),
                &format!("Applied: {}", applied_rule.description),
            ).await;
        }
        
        // Log overall pricing result
        self.audit_logger.log_pricing_application(
            order_id,
            None,
            rule_data,
            &effect,
        ).await;
        
        Ok(result)
    }
    
    /// Estimate preparation time for an order
    /// 
    /// Calculates prep time based on items and current queue.
    pub async fn estimate_prep_time(
        &self,
        items: &[PrepTimeOrderItem],
    ) -> BRResult<PrepTimeEstimate> {
        let _timer = self.metrics.start_prep_time_estimate();
        
        self.prep_time_calculator.estimate(items).await
    }
    
    /// Award loyalty points for an order
    /// 
    /// Calculates and awards loyalty points, then logs the award.
    pub async fn award_loyalty_points(
        &self,
        order_id: Uuid,
        customer_id: i32,
        order_total: rust_decimal::Decimal,
        items: &[LoyaltyOrderItem],
    ) -> BRResult<i32> {
        let _timer = self.metrics.start_loyalty_calculation();
        
        // Calculate points
        let calculation = self.loyalty_engine.calculate_points(order_total, items).await?;
        
        // Award points
        let customer_loyalty = self.loyalty_engine.award_points(customer_id, calculation.total_points).await?;
        
        // Log loyalty award
        let rule_data = json!({
            "customer_id": customer_id,
            "order_total": order_total,
            "base_points": calculation.base_points,
            "bonus_points": calculation.bonus_points,
            "total_points": calculation.total_points,
            "new_balance": customer_loyalty.points_balance,
            "lifetime_points": customer_loyalty.lifetime_points,
        });
        
        let effect = format!(
            "Awarded {} points (base: {}, bonus: {})",
            calculation.total_points,
            calculation.base_points,
            calculation.bonus_points
        );
        
        self.audit_logger.log_loyalty_award(order_id, rule_data, &effect).await;
        
        Ok(calculation.total_points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_business_rules_engine_structure() {
        // Test that we can describe the engine structure
        // Actual instantiation requires a database pool
        
        // Verify all component types are accessible
        let _availability_type: Option<AvailabilityEngine> = None;
        let _pricing_type: Option<PricingEngine> = None;
        let _prep_time_type: Option<PrepTimeCalculator> = None;
        let _loyalty_type: Option<LoyaltyEngine> = None;
        let _audit_type: Option<AuditLogger> = None;
    }
    
    #[test]
    fn test_order_item_types() {
        // Test that all order item types are compatible
        let _availability_item: Option<OrderItem> = None;
        let _pricing_item: Option<PricingOrderItem> = None;
        let _prep_time_item: Option<PrepTimeOrderItem> = None;
        let _loyalty_item: Option<LoyaltyOrderItem> = None;
    }
    
    #[test]
    fn test_result_types() {
        // Test that all result types are accessible
        let _validation_result: Option<OrderValidationResult> = None;
        let _pricing_result: Option<OrderPricingResult> = None;
        let _prep_time_result: Option<PrepTimeEstimate> = None;
        let _loyalty_result: Option<CustomerLoyalty> = None;
    }
}
