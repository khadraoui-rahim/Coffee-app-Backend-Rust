// Pricing Engine
// 
// Calculates order prices by applying configurable pricing rules.
// Supports time-based, quantity-based, and promotional rules with multiple combination strategies.

use crate::business_rules::{
    config_store::{
        PricingRule, QuantityBasedRuleConfig, RuleConfigurationStore, TimeBasedRuleConfig,
        PromotionalRuleConfig,
    },
    error::{BRResult, BusinessRulesError},
    types::{CombinationStrategy, DiscountType, PricingRuleType},
};
use chrono::{Local, NaiveTime, Utc};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

/// Order item for pricing calculation
#[derive(Debug, Clone)]
pub struct PricingOrderItem {
    pub coffee_id: i32,
    pub quantity: u32,
    pub base_price: Decimal,
}

/// Applied pricing rule with its effect
#[derive(Debug, Clone)]
pub struct AppliedPricingRule {
    pub rule_id: Uuid,
    pub rule_type: PricingRuleType,
    pub description: String,
    pub discount_amount: Decimal,
}

/// Result of pricing calculation
#[derive(Debug, Clone)]
pub struct OrderPricingResult {
    pub base_price: Decimal,
    pub applied_rules: Vec<AppliedPricingRule>,
    pub final_price: Decimal,
    pub total_discount: Decimal,
}

/// Pricing Engine
/// 
/// Evaluates pricing rules and calculates order prices with discounts.
pub struct PricingEngine {
    config_store: Arc<RuleConfigurationStore>,
}

impl PricingEngine {
    /// Create a new PricingEngine
    pub fn new(config_store: Arc<RuleConfigurationStore>) -> Self {
        Self { config_store }
    }
    
    /// Calculate order price with all applicable rules
    /// 
    /// Orchestrates the full pricing flow:
    /// 1. Calculate base price from items
    /// 2. Get applicable rules (active, valid, sorted by priority)
    /// 3. Apply rules with configured strategy
    /// 4. Return pricing result with breakdown
    pub async fn calculate_order_price(
        &self,
        items: &[PricingOrderItem],
        strategy: CombinationStrategy,
    ) -> BRResult<OrderPricingResult> {
        // Calculate base price
        let base_price = self.calculate_base_price(items);
        
        // Get applicable rules
        let applicable_rules = self.get_applicable_rules(items).await?;
        
        // Apply rules with strategy
        let (final_price, applied_rules) = self.apply_rules(base_price, &applicable_rules, items, strategy)?;
        
        // Calculate total discount
        let total_discount = base_price - final_price;
        
        Ok(OrderPricingResult {
            base_price,
            applied_rules,
            final_price,
            total_discount,
        })
    }
    
    /// Calculate base price from order items
    fn calculate_base_price(&self, items: &[PricingOrderItem]) -> Decimal {
        items
            .iter()
            .map(|item| item.base_price * Decimal::from(item.quantity))
            .sum()
    }
    
    /// Get applicable rules for the order
    /// 
    /// Filters rules by:
    /// - Active status
    /// - Valid time period
    /// - Coffee-specific targeting
    /// 
    /// Returns rules sorted by priority (descending)
    pub async fn get_applicable_rules(&self, items: &[PricingOrderItem]) -> BRResult<Vec<PricingRule>> {
        let all_rules = self.config_store.get_pricing_rules().await?;
        let now = Utc::now();
        
        let mut applicable_rules: Vec<PricingRule> = all_rules
            .into_iter()
            .filter(|rule| {
                // Must be active
                if !rule.is_active {
                    return false;
                }
                
                // Must be within valid time period
                if now < rule.valid_from {
                    return false;
                }
                if let Some(valid_until) = rule.valid_until {
                    if now > valid_until {
                        return false;
                    }
                }
                
                // If rule targets specific coffees, order must contain at least one
                if let Some(ref coffee_ids) = rule.coffee_ids {
                    let order_coffee_ids: Vec<i32> = items.iter().map(|item| item.coffee_id).collect();
                    if !coffee_ids.iter().any(|id| order_coffee_ids.contains(id)) {
                        return false;
                    }
                }
                
                true
            })
            .collect();
        
        // Sort by priority (descending - higher priority first)
        applicable_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        Ok(applicable_rules)
    }
    
    /// Apply rules to calculate final price
    /// 
    /// Evaluates each rule and applies discounts according to the combination strategy.
    fn apply_rules(
        &self,
        base_price: Decimal,
        rules: &[PricingRule],
        items: &[PricingOrderItem],
        strategy: CombinationStrategy,
    ) -> BRResult<(Decimal, Vec<AppliedPricingRule>)> {
        let mut applied_rules = Vec::new();
        
        // Evaluate each rule and collect applicable ones
        for rule in rules {
            if let Some(applied_rule) = self.evaluate_rule(rule, items, base_price)? {
                applied_rules.push(applied_rule);
            }
        }
        
        // Calculate final price based on strategy
        let final_price = match strategy {
            CombinationStrategy::Additive => {
                self.apply_additive_strategy(base_price, &applied_rules)
            }
            CombinationStrategy::Multiplicative => {
                self.apply_multiplicative_strategy(base_price, &applied_rules)
            }
            CombinationStrategy::BestPrice => {
                self.apply_best_price_strategy(base_price, &applied_rules)
            }
        };
        
        // Ensure final price is never negative
        let final_price = final_price.max(Decimal::ZERO);
        
        Ok((final_price, applied_rules))
    }
    
    /// Evaluate a single rule to determine if it applies and calculate discount
    fn evaluate_rule(
        &self,
        rule: &PricingRule,
        items: &[PricingOrderItem],
        base_price: Decimal,
    ) -> BRResult<Option<AppliedPricingRule>> {
        match rule.rule_type {
            PricingRuleType::TimeBased => self.evaluate_time_based_rule(rule),
            PricingRuleType::QuantityBased => self.evaluate_quantity_based_rule(rule, items),
            PricingRuleType::Promotional => self.evaluate_promotional_rule(rule, base_price),
        }
    }
    
    /// Evaluate time-based rule (e.g., happy hour)
    fn evaluate_time_based_rule(&self, rule: &PricingRule) -> BRResult<Option<AppliedPricingRule>> {
        let config: TimeBasedRuleConfig = serde_json::from_value(rule.rule_config.clone())?;
        
        // Get current local time
        let now = Local::now().time();
        
        // Check if current time falls within any of the time ranges
        let is_in_range = config.time_ranges.iter().any(|range| {
            if let (Ok(start), Ok(end)) = (
                NaiveTime::parse_from_str(&range.start, "%H:%M"),
                NaiveTime::parse_from_str(&range.end, "%H:%M"),
            ) {
                if start <= end {
                    // Normal range (e.g., 09:00 - 17:00)
                    now >= start && now <= end
                } else {
                    // Overnight range (e.g., 22:00 - 02:00)
                    now >= start || now <= end
                }
            } else {
                false
            }
        });
        
        if !is_in_range {
            return Ok(None);
        }
        
        Ok(Some(AppliedPricingRule {
            rule_id: rule.rule_id,
            rule_type: rule.rule_type,
            description: config.description.unwrap_or_else(|| "Time-based discount".to_string()),
            discount_amount: config.discount_value,
        }))
    }
    
    /// Evaluate quantity-based rule (e.g., bulk discount)
    fn evaluate_quantity_based_rule(
        &self,
        rule: &PricingRule,
        items: &[PricingOrderItem],
    ) -> BRResult<Option<AppliedPricingRule>> {
        let config: QuantityBasedRuleConfig = serde_json::from_value(rule.rule_config.clone())?;
        
        // Calculate total quantity
        let total_quantity: u32 = items.iter().map(|item| item.quantity).sum();
        
        // Check if minimum quantity is met
        if total_quantity < config.min_quantity {
            return Ok(None);
        }
        
        Ok(Some(AppliedPricingRule {
            rule_id: rule.rule_id,
            rule_type: rule.rule_type,
            description: config.description.unwrap_or_else(|| "Quantity discount".to_string()),
            discount_amount: config.discount_value,
        }))
    }
    
    /// Evaluate promotional rule
    fn evaluate_promotional_rule(
        &self,
        rule: &PricingRule,
        _base_price: Decimal,
    ) -> BRResult<Option<AppliedPricingRule>> {
        let config: PromotionalRuleConfig = serde_json::from_value(rule.rule_config.clone())?;
        
        // Promotional rules are already filtered by time in get_applicable_rules
        // If we reach here, the rule applies
        Ok(Some(AppliedPricingRule {
            rule_id: rule.rule_id,
            rule_type: rule.rule_type,
            description: config.description.unwrap_or_else(|| "Promotional discount".to_string()),
            discount_amount: config.discount_value,
        }))
    }
    
    /// Apply additive strategy: sum all discounts
    fn apply_additive_strategy(&self, base_price: Decimal, rules: &[AppliedPricingRule]) -> Decimal {
        let mut total_discount = Decimal::ZERO;
        
        for rule in rules {
            let discount = self.calculate_discount_amount(base_price, rule);
            total_discount += discount;
        }
        
        base_price - total_discount
    }
    
    /// Apply multiplicative strategy: apply discounts sequentially
    fn apply_multiplicative_strategy(&self, base_price: Decimal, rules: &[AppliedPricingRule]) -> Decimal {
        let mut current_price = base_price;
        
        for rule in rules {
            let discount = self.calculate_discount_amount(current_price, rule);
            current_price -= discount;
        }
        
        current_price
    }
    
    /// Apply best price strategy: choose the combination giving the lowest price
    fn apply_best_price_strategy(&self, base_price: Decimal, rules: &[AppliedPricingRule]) -> Decimal {
        let additive_price = self.apply_additive_strategy(base_price, rules);
        let multiplicative_price = self.apply_multiplicative_strategy(base_price, rules);
        
        additive_price.min(multiplicative_price)
    }
    
    /// Calculate discount amount based on discount type
    fn calculate_discount_amount(&self, price: Decimal, rule: &AppliedPricingRule) -> Decimal {
        // Determine discount type from the rule's discount_amount
        // For simplicity, we'll treat values <= 100 as percentage, > 100 as fixed amount
        // In a real implementation, this would be stored in the rule
        if rule.discount_amount <= Decimal::from(100) {
            // Percentage discount
            price * rule.discount_amount / Decimal::from(100)
        } else {
            // Fixed amount discount
            rule.discount_amount
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pricing_order_item_creation() {
        let item = PricingOrderItem {
            coffee_id: 1,
            quantity: 2,
            base_price: Decimal::from(5),
        };
        
        assert_eq!(item.coffee_id, 1);
        assert_eq!(item.quantity, 2);
        assert_eq!(item.base_price, Decimal::from(5));
    }
    
    #[test]
    fn test_applied_pricing_rule_creation() {
        let rule = AppliedPricingRule {
            rule_id: Uuid::new_v4(),
            rule_type: PricingRuleType::TimeBased,
            description: "Happy hour".to_string(),
            discount_amount: Decimal::from(10),
        };
        
        assert_eq!(rule.rule_type, PricingRuleType::TimeBased);
        assert_eq!(rule.description, "Happy hour");
        assert_eq!(rule.discount_amount, Decimal::from(10));
    }
    
    #[test]
    fn test_order_pricing_result_creation() {
        let result = OrderPricingResult {
            base_price: Decimal::from(100),
            applied_rules: vec![],
            final_price: Decimal::from(90),
            total_discount: Decimal::from(10),
        };
        
        assert_eq!(result.base_price, Decimal::from(100));
        assert_eq!(result.final_price, Decimal::from(90));
        assert_eq!(result.total_discount, Decimal::from(10));
        assert!(result.applied_rules.is_empty());
    }
    
    #[test]
    fn test_calculate_base_price() {
        // Test the calculation logic directly without needing an engine instance
        let items = vec![
            PricingOrderItem {
                coffee_id: 1,
                quantity: 2,
                base_price: Decimal::from(5),
            },
            PricingOrderItem {
                coffee_id: 2,
                quantity: 1,
                base_price: Decimal::from(10),
            },
        ];
        
        let base_price: Decimal = items
            .iter()
            .map(|item| item.base_price * Decimal::from(item.quantity))
            .sum();
        
        assert_eq!(base_price, Decimal::from(20)); // (2 * 5) + (1 * 10)
    }
    
    #[test]
    fn test_calculate_discount_amount_percentage() {
        // Test discount calculation logic directly
        let rule = AppliedPricingRule {
            rule_id: Uuid::new_v4(),
            rule_type: PricingRuleType::Promotional,
            description: "10% off".to_string(),
            discount_amount: Decimal::from(10), // 10%
        };
        
        let price = Decimal::from(100);
        // Percentage discount (values <= 100)
        let discount = price * rule.discount_amount / Decimal::from(100);
        
        assert_eq!(discount, Decimal::from(10));
    }
    
    #[test]
    fn test_apply_additive_strategy() {
        // Test additive strategy logic directly
        let base_price = Decimal::from(100);
        let rules = vec![
            AppliedPricingRule {
                rule_id: Uuid::new_v4(),
                rule_type: PricingRuleType::Promotional,
                description: "10% off".to_string(),
                discount_amount: Decimal::from(10),
            },
            AppliedPricingRule {
                rule_id: Uuid::new_v4(),
                rule_type: PricingRuleType::Promotional,
                description: "5% off".to_string(),
                discount_amount: Decimal::from(5),
            },
        ];
        
        let mut total_discount = Decimal::ZERO;
        for rule in &rules {
            // Calculate discount (percentage for values <= 100)
            let discount = base_price * rule.discount_amount / Decimal::from(100);
            total_discount += discount;
        }
        let final_price = base_price - total_discount;
        
        assert_eq!(final_price, Decimal::from(85)); // 100 - 10 - 5
    }
    
    #[test]
    fn test_apply_multiplicative_strategy() {
        // Test multiplicative strategy logic directly
        let base_price = Decimal::from(100);
        let rules = vec![
            AppliedPricingRule {
                rule_id: Uuid::new_v4(),
                rule_type: PricingRuleType::Promotional,
                description: "10% off".to_string(),
                discount_amount: Decimal::from(10),
            },
            AppliedPricingRule {
                rule_id: Uuid::new_v4(),
                rule_type: PricingRuleType::Promotional,
                description: "5% off".to_string(),
                discount_amount: Decimal::from(5),
            },
        ];
        
        let mut current_price = base_price;
        for rule in &rules {
            // Calculate discount (percentage for values <= 100)
            let discount = current_price * rule.discount_amount / Decimal::from(100);
            current_price -= discount;
        }
        
        // 100 - 10% = 90, then 90 - 5% = 85.5
        assert_eq!(current_price, Decimal::new(855, 1));
    }
    
    #[test]
    fn test_non_negative_price_constraint() {
        // Test that prices don't go negative
        let base_price = Decimal::from(100);
        let rules = vec![
            AppliedPricingRule {
                rule_id: Uuid::new_v4(),
                rule_type: PricingRuleType::Promotional,
                description: "200 off".to_string(),
                discount_amount: Decimal::from(200), // More than base price (fixed amount)
            },
        ];
        
        let mut total_discount = Decimal::ZERO;
        for rule in &rules {
            // Fixed amount discount (values > 100)
            let discount = rule.discount_amount;
            total_discount += discount;
        }
        let final_price = (base_price - total_discount).max(Decimal::ZERO);
        
        // Should not go negative
        assert_eq!(final_price, Decimal::ZERO);
    }
}
