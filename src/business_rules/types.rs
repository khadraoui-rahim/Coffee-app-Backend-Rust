// Domain type definitions for the Business Rules System
// Provides shared types used across multiple engines

use serde::{Deserialize, Serialize};
use std::fmt;

/// Availability status for coffee items
/// 
/// Represents the different states a coffee item can be in regarding availability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AvailabilityStatus {
    /// Coffee is available for ordering
    Available,
    
    /// Coffee is temporarily out of stock
    OutOfStock,
    
    /// Coffee is only available during certain seasons
    Seasonal,
    
    /// Coffee has been permanently discontinued
    Discontinued,
}

impl fmt::Display for AvailabilityStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AvailabilityStatus::Available => write!(f, "available"),
            AvailabilityStatus::OutOfStock => write!(f, "out_of_stock"),
            AvailabilityStatus::Seasonal => write!(f, "seasonal"),
            AvailabilityStatus::Discontinued => write!(f, "discontinued"),
        }
    }
}

impl std::str::FromStr for AvailabilityStatus {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "available" => Ok(AvailabilityStatus::Available),
            "out_of_stock" => Ok(AvailabilityStatus::OutOfStock),
            "seasonal" => Ok(AvailabilityStatus::Seasonal),
            "discontinued" => Ok(AvailabilityStatus::Discontinued),
            _ => Err(format!("Invalid availability status: {}", s)),
        }
    }
}

/// Type of discount applied by pricing rules
/// 
/// Determines how the discount value should be interpreted and applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscountType {
    /// Discount is a percentage of the price (e.g., 10 = 10% off)
    Percentage,
    
    /// Discount is a fixed amount subtracted from the price (e.g., 5.00 = $5 off)
    FixedAmount,
}

impl fmt::Display for DiscountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiscountType::Percentage => write!(f, "percentage"),
            DiscountType::FixedAmount => write!(f, "fixed_amount"),
        }
    }
}

/// Strategy for combining multiple pricing rules
/// 
/// When multiple pricing rules apply to an order, this determines how
/// the discounts are combined to calculate the final price.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CombinationStrategy {
    /// Add all discounts together (sum of all discount amounts)
    /// Example: 10% + 5% = 15% total discount
    Additive,
    
    /// Apply discounts sequentially (multiplicative)
    /// Example: 10% then 5% = 14.5% total discount (not 15%)
    Multiplicative,
    
    /// Choose the combination that gives the best price for the customer
    /// Evaluates both additive and multiplicative, returns the lower price
    BestPrice,
}

impl fmt::Display for CombinationStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CombinationStrategy::Additive => write!(f, "additive"),
            CombinationStrategy::Multiplicative => write!(f, "multiplicative"),
            CombinationStrategy::BestPrice => write!(f, "best_price"),
        }
    }
}

impl Default for CombinationStrategy {
    fn default() -> Self {
        CombinationStrategy::BestPrice
    }
}

/// Type of pricing rule
/// 
/// Categorizes pricing rules by their evaluation criteria.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PricingRuleType {
    /// Rule based on time of day/week (e.g., happy hour)
    TimeBased,
    
    /// Rule based on quantity of items (e.g., bulk discount)
    QuantityBased,
    
    /// Promotional rule with specific validity period
    Promotional,
}

impl fmt::Display for PricingRuleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PricingRuleType::TimeBased => write!(f, "time_based"),
            PricingRuleType::QuantityBased => write!(f, "quantity_based"),
            PricingRuleType::Promotional => write!(f, "promotional"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_availability_status_display() {
        assert_eq!(AvailabilityStatus::Available.to_string(), "available");
        assert_eq!(AvailabilityStatus::OutOfStock.to_string(), "out_of_stock");
        assert_eq!(AvailabilityStatus::Seasonal.to_string(), "seasonal");
        assert_eq!(AvailabilityStatus::Discontinued.to_string(), "discontinued");
    }
    
    #[test]
    fn test_availability_status_from_str() {
        use std::str::FromStr;
        
        assert_eq!(
            AvailabilityStatus::from_str("available").unwrap(),
            AvailabilityStatus::Available
        );
        assert_eq!(
            AvailabilityStatus::from_str("out_of_stock").unwrap(),
            AvailabilityStatus::OutOfStock
        );
        assert!(AvailabilityStatus::from_str("invalid").is_err());
    }
    
    #[test]
    fn test_discount_type_display() {
        assert_eq!(DiscountType::Percentage.to_string(), "percentage");
        assert_eq!(DiscountType::FixedAmount.to_string(), "fixed_amount");
    }
    
    #[test]
    fn test_combination_strategy_display() {
        assert_eq!(CombinationStrategy::Additive.to_string(), "additive");
        assert_eq!(CombinationStrategy::Multiplicative.to_string(), "multiplicative");
        assert_eq!(CombinationStrategy::BestPrice.to_string(), "best_price");
    }
    
    #[test]
    fn test_combination_strategy_default() {
        assert_eq!(CombinationStrategy::default(), CombinationStrategy::BestPrice);
    }
    
    #[test]
    fn test_pricing_rule_type_display() {
        assert_eq!(PricingRuleType::TimeBased.to_string(), "time_based");
        assert_eq!(PricingRuleType::QuantityBased.to_string(), "quantity_based");
        assert_eq!(PricingRuleType::Promotional.to_string(), "promotional");
    }
    
    #[test]
    fn test_serialization() {
        // Test that types can be serialized to JSON
        let status = AvailabilityStatus::Available;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"available\"");
        
        let discount = DiscountType::Percentage;
        let json = serde_json::to_string(&discount).unwrap();
        assert_eq!(json, "\"percentage\"");
        
        let strategy = CombinationStrategy::Additive;
        let json = serde_json::to_string(&strategy).unwrap();
        assert_eq!(json, "\"additive\"");
    }
    
    #[test]
    fn test_deserialization() {
        // Test that types can be deserialized from JSON
        let status: AvailabilityStatus = serde_json::from_str("\"available\"").unwrap();
        assert_eq!(status, AvailabilityStatus::Available);
        
        let discount: DiscountType = serde_json::from_str("\"percentage\"").unwrap();
        assert_eq!(discount, DiscountType::Percentage);
        
        let strategy: CombinationStrategy = serde_json::from_str("\"additive\"").unwrap();
        assert_eq!(strategy, CombinationStrategy::Additive);
    }
}
