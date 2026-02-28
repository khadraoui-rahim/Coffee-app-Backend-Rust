// Rule Configuration Store
// 
// Manages loading, caching, and validation of business rule configurations from the database.
// Implements a time-based cache with 60-second TTL to balance performance and freshness.

use crate::business_rules::{
    error::{BRResult, BusinessRulesError},
    types::{AvailabilityStatus, DiscountType, PricingRuleType},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use rust_decimal::Decimal;
use uuid::Uuid;

/// Time-to-live for cached configurations (60 seconds)
const CACHE_TTL: Duration = Duration::from_secs(60);

/// Coffee availability configuration from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoffeeAvailability {
    pub coffee_id: i32,
    pub status: AvailabilityStatus,
    pub reason: Option<String>,
    pub available_from: Option<DateTime<Utc>>,
    pub available_until: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

/// Pricing rule configuration from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingRule {
    pub rule_id: Uuid,
    pub rule_type: PricingRuleType,
    pub priority: i32,
    pub rule_config: serde_json::Value,
    pub coffee_ids: Option<Vec<i32>>,
    pub is_active: bool,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
}

/// Time-based pricing rule details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBasedRuleConfig {
    pub time_ranges: Vec<TimeRange>,
    pub discount_type: DiscountType,
    pub discount_value: Decimal,
    pub description: Option<String>,
}

/// Time range for time-based rules (HH:MM format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: String, // Format: "HH:MM"
    pub end: String,   // Format: "HH:MM"
}

/// Quantity-based pricing rule details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantityBasedRuleConfig {
    pub min_quantity: u32,
    pub discount_type: DiscountType,
    pub discount_value: Decimal,
    pub description: Option<String>,
}

/// Promotional pricing rule details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionalRuleConfig {
    pub discount_type: DiscountType,
    pub discount_value: Decimal,
    pub description: Option<String>,
}

/// Preparation time configuration for a coffee item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoffeeBaseTime {
    pub coffee_id: i32,
    pub base_minutes: i32,
    pub per_additional_item: i32,
    pub updated_at: DateTime<Utc>,
}

/// Loyalty program configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyConfig {
    pub config_id: i32,
    pub points_per_dollar: Decimal,
    pub bonus_multipliers: HashMap<i32, Decimal>,
    pub updated_at: DateTime<Utc>,
}

/// In-memory cache for rule configurations
#[derive(Debug, Clone)]
struct ConfigCache {
    availability_rules: HashMap<i32, CoffeeAvailability>,
    pricing_rules: Vec<PricingRule>,
    prep_time_config: HashMap<i32, CoffeeBaseTime>,
    loyalty_config: Option<LoyaltyConfig>,
    last_updated: HashMap<String, Instant>,
}

impl ConfigCache {
    fn new() -> Self {
        Self {
            availability_rules: HashMap::new(),
            pricing_rules: Vec::new(),
            prep_time_config: HashMap::new(),
            loyalty_config: None,
            last_updated: HashMap::new(),
        }
    }
    
    fn is_stale(&self, rule_type: &str, ttl: Duration) -> bool {
        match self.last_updated.get(rule_type) {
            Some(last_update) => last_update.elapsed() > ttl,
            None => true, // Never loaded, so it's stale
        }
    }
    
    fn mark_updated(&mut self, rule_type: &str) {
        self.last_updated.insert(rule_type.to_string(), Instant::now());
    }
}

/// Rule Configuration Store
/// 
/// Manages loading and caching of business rule configurations from PostgreSQL.
/// Implements a time-based cache with automatic refresh when data becomes stale.
pub struct RuleConfigurationStore {
    pool: PgPool,
    cache: Arc<RwLock<ConfigCache>>,
    cache_ttl: Duration,
    metrics: Option<Arc<crate::business_rules::metrics::PerformanceMetrics>>,
}

impl RuleConfigurationStore {
    /// Create a new RuleConfigurationStore
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            cache: Arc::new(RwLock::new(ConfigCache::new())),
            cache_ttl: CACHE_TTL,
            metrics: None,
        }
    }
    
    /// Create a new RuleConfigurationStore with metrics tracking
    pub fn with_metrics(
        pool: PgPool,
        metrics: Arc<crate::business_rules::metrics::PerformanceMetrics>,
    ) -> Self {
        Self {
            pool,
            cache: Arc::new(RwLock::new(ConfigCache::new())),
            cache_ttl: CACHE_TTL,
            metrics: Some(metrics),
        }
    }
    
    /// Record a cache hit
    fn record_cache_hit(&self) {
        if let Some(ref metrics) = self.metrics {
            metrics.record_cache_hit();
        }
    }
    
    /// Record a cache miss
    fn record_cache_miss(&self) {
        if let Some(ref metrics) = self.metrics {
            metrics.record_cache_miss();
        }
    }
    
    /// Get a reference to the database pool
    /// 
    /// Used by engines that need to perform database operations.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
    
    /// Load availability rules from database
    /// 
    /// Queries the coffee_availability table and returns a map of coffee_id to availability status.
    pub async fn load_availability_rules(&self) -> BRResult<HashMap<i32, CoffeeAvailability>> {
        let rules = sqlx::query_as!(
            CoffeeAvailability,
            r#"
            SELECT 
                coffee_id,
                status as "status: AvailabilityStatus",
                reason,
                available_from,
                available_until,
                updated_at
            FROM coffee_availability
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut map = HashMap::new();
        for rule in rules {
            map.insert(rule.coffee_id, rule);
        }
        
        Ok(map)
    }
    
    /// Load pricing rules from database
    /// 
    /// Queries the pricing_rules table and parses JSON configurations.
    /// Only returns active rules.
    pub async fn load_pricing_rules(&self) -> BRResult<Vec<PricingRule>> {
        let rules = sqlx::query_as!(
            PricingRule,
            r#"
            SELECT 
                rule_id,
                rule_type as "rule_type: PricingRuleType",
                priority,
                rule_config,
                coffee_ids,
                is_active,
                valid_from,
                valid_until
            FROM pricing_rules
            WHERE is_active = true
            ORDER BY priority DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        // Validate each rule's JSON configuration
        for rule in &rules {
            self.validate_pricing_rule(rule)?;
        }
        
        Ok(rules)
    }
    
    /// Load preparation time configuration from database
    /// 
    /// Queries the prep_time_config table and returns a map of coffee_id to prep time settings.
    pub async fn load_prep_time_config(&self) -> BRResult<HashMap<i32, CoffeeBaseTime>> {
        let configs = sqlx::query_as!(
            CoffeeBaseTime,
            r#"
            SELECT 
                coffee_id,
                base_minutes,
                per_additional_item,
                updated_at
            FROM prep_time_config
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        // Validate prep time values
        for config in &configs {
            if config.base_minutes <= 0 {
                return Err(BusinessRulesError::InvalidConfiguration(
                    format!("Invalid base_minutes for coffee {}: must be positive", config.coffee_id)
                ));
            }
            if config.per_additional_item < 0 {
                return Err(BusinessRulesError::InvalidConfiguration(
                    format!("Invalid per_additional_item for coffee {}: must be non-negative", config.coffee_id)
                ));
            }
        }
        
        let mut map = HashMap::new();
        for config in configs {
            map.insert(config.coffee_id, config);
        }
        
        Ok(map)
    }
    
    /// Load loyalty configuration from database
    /// 
    /// Queries the loyalty_config table (singleton) and parses bonus multipliers from JSONB.
    pub async fn load_loyalty_config(&self) -> BRResult<LoyaltyConfig> {
        let config = sqlx::query!(
            r#"
            SELECT 
                config_id,
                points_per_dollar,
                bonus_multipliers,
                updated_at
            FROM loyalty_config
            WHERE config_id = 1
            "#
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| BusinessRulesError::ConfigurationNotFound("loyalty_config".to_string()))?;
        
        // Parse bonus multipliers from JSONB
        let bonus_multipliers: HashMap<i32, Decimal> = serde_json::from_value(config.bonus_multipliers)
            .map_err(|e| BusinessRulesError::InvalidConfiguration(
                format!("Invalid bonus_multipliers JSON: {}", e)
            ))?;
        
        // Validate loyalty config values
        if config.points_per_dollar < Decimal::ZERO {
            return Err(BusinessRulesError::InvalidConfiguration(
                "points_per_dollar must be non-negative".to_string()
            ));
        }
        
        for (coffee_id, multiplier) in &bonus_multipliers {
            if *multiplier < Decimal::ZERO {
                return Err(BusinessRulesError::InvalidConfiguration(
                    format!("Bonus multiplier for coffee {} must be non-negative", coffee_id)
                ));
            }
        }
        
        Ok(LoyaltyConfig {
            config_id: config.config_id,
            points_per_dollar: config.points_per_dollar,
            bonus_multipliers,
            updated_at: config.updated_at,
        })
    }
    
    /// Get availability rules with caching
    /// 
    /// Returns cached data if fresh, otherwise reloads from database.
    pub async fn get_availability_rules(&self) -> BRResult<HashMap<i32, CoffeeAvailability>> {
        self.refresh_if_stale("availability").await?;
        
        let cache = self.cache.read().await;
        Ok(cache.availability_rules.clone())
    }
    
    /// Get pricing rules with caching
    /// 
    /// Returns cached data if fresh, otherwise reloads from database.
    pub async fn get_pricing_rules(&self) -> BRResult<Vec<PricingRule>> {
        self.refresh_if_stale("pricing").await?;
        
        let cache = self.cache.read().await;
        Ok(cache.pricing_rules.clone())
    }
    
    /// Get prep time configuration with caching
    /// 
    /// Returns cached data if fresh, otherwise reloads from database.
    pub async fn get_prep_time_config(&self) -> BRResult<HashMap<i32, CoffeeBaseTime>> {
        self.refresh_if_stale("prep_time").await?;
        
        let cache = self.cache.read().await;
        Ok(cache.prep_time_config.clone())
    }
    
    /// Get loyalty configuration with caching
    /// 
    /// Returns cached data if fresh, otherwise reloads from database.
    pub async fn get_loyalty_config(&self) -> BRResult<LoyaltyConfig> {
        self.refresh_if_stale("loyalty").await?;
        
        let cache = self.cache.read().await;
        cache.loyalty_config.clone()
            .ok_or_else(|| BusinessRulesError::ConfigurationNotFound("loyalty_config".to_string()))
    }
    
    /// Refresh cache if data is stale
    /// 
    /// Checks the last update time and reloads from database if TTL has expired.
    async fn refresh_if_stale(&self, rule_type: &str) -> BRResult<()> {
        // Check if stale with read lock first (fast path)
        {
            let cache = self.cache.read().await;
            if !cache.is_stale(rule_type, self.cache_ttl) {
                self.record_cache_hit();
                return Ok(());
            }
        }
        
        // Cache miss - need to refresh
        self.record_cache_miss();
        
        // Need to refresh - acquire write lock
        let mut cache = self.cache.write().await;
        
        // Double-check after acquiring write lock (another thread might have refreshed)
        if !cache.is_stale(rule_type, self.cache_ttl) {
            return Ok(());
        }
        
        // Load fresh data from database
        match rule_type {
            "availability" => {
                let rules = self.load_availability_rules().await?;
                cache.availability_rules = rules;
                cache.mark_updated("availability");
            }
            "pricing" => {
                let rules = self.load_pricing_rules().await?;
                cache.pricing_rules = rules;
                cache.mark_updated("pricing");
            }
            "prep_time" => {
                let config = self.load_prep_time_config().await?;
                cache.prep_time_config = config;
                cache.mark_updated("prep_time");
            }
            "loyalty" => {
                let config = self.load_loyalty_config().await?;
                cache.loyalty_config = Some(config);
                cache.mark_updated("loyalty");
            }
            _ => {
                return Err(BusinessRulesError::InvalidConfiguration(
                    format!("Unknown rule type: {}", rule_type)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Invalidate cache for a specific rule type
    /// 
    /// Forces the next access to reload from database.
    pub async fn invalidate_cache(&self, rule_type: &str) {
        let mut cache = self.cache.write().await;
        cache.last_updated.remove(rule_type);
    }
    
    /// Validate pricing rule JSON configuration
    /// 
    /// Ensures the rule_config JSON matches the expected structure for the rule type.
    fn validate_pricing_rule(&self, rule: &PricingRule) -> BRResult<()> {
        match rule.rule_type {
            PricingRuleType::TimeBased => {
                let config: TimeBasedRuleConfig = serde_json::from_value(rule.rule_config.clone())
                    .map_err(|e| BusinessRulesError::InvalidPricingRule(
                        format!("Invalid time_based rule config: {}", e)
                    ))?;
                
                // Validate time ranges
                for time_range in &config.time_ranges {
                    self.validate_time_format(&time_range.start)?;
                    self.validate_time_format(&time_range.end)?;
                }
                
                // Validate discount value
                self.validate_discount_value(&config.discount_type, config.discount_value)?;
            }
            PricingRuleType::QuantityBased => {
                let config: QuantityBasedRuleConfig = serde_json::from_value(rule.rule_config.clone())
                    .map_err(|e| BusinessRulesError::InvalidPricingRule(
                        format!("Invalid quantity_based rule config: {}", e)
                    ))?;
                
                // Validate min_quantity
                if config.min_quantity == 0 {
                    return Err(BusinessRulesError::InvalidPricingRule(
                        "min_quantity must be greater than 0".to_string()
                    ));
                }
                
                // Validate discount value
                self.validate_discount_value(&config.discount_type, config.discount_value)?;
            }
            PricingRuleType::Promotional => {
                let config: PromotionalRuleConfig = serde_json::from_value(rule.rule_config.clone())
                    .map_err(|e| BusinessRulesError::InvalidPricingRule(
                        format!("Invalid promotional rule config: {}", e)
                    ))?;
                
                // Validate discount value
                self.validate_discount_value(&config.discount_type, config.discount_value)?;
            }
        }
        
        Ok(())
    }
    
    /// Validate time format (HH:MM)
    fn validate_time_format(&self, time_str: &str) -> BRResult<()> {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 2 {
            return Err(BusinessRulesError::InvalidPricingRule(
                format!("Invalid time format '{}': expected HH:MM", time_str)
            ));
        }
        
        let hour: u32 = parts[0].parse()
            .map_err(|_| BusinessRulesError::InvalidPricingRule(
                format!("Invalid hour in time '{}'", time_str)
            ))?;
        let minute: u32 = parts[1].parse()
            .map_err(|_| BusinessRulesError::InvalidPricingRule(
                format!("Invalid minute in time '{}'", time_str)
            ))?;
        
        if hour >= 24 {
            return Err(BusinessRulesError::InvalidPricingRule(
                format!("Hour must be 0-23 in time '{}'", time_str)
            ));
        }
        if minute >= 60 {
            return Err(BusinessRulesError::InvalidPricingRule(
                format!("Minute must be 0-59 in time '{}'", time_str)
            ));
        }
        
        Ok(())
    }
    
    /// Validate discount value based on discount type
    fn validate_discount_value(&self, discount_type: &DiscountType, value: Decimal) -> BRResult<()> {
        if value < Decimal::ZERO {
            return Err(BusinessRulesError::InvalidPricingRule(
                "Discount value must be non-negative".to_string()
            ));
        }
        
        match discount_type {
            DiscountType::Percentage => {
                if value > Decimal::from(100) {
                    return Err(BusinessRulesError::InvalidPricingRule(
                        "Percentage discount cannot exceed 100%".to_string()
                    ));
                }
            }
            DiscountType::FixedAmount => {
                // Fixed amount can be any non-negative value
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_cache_is_stale() {
        let mut cache = ConfigCache::new();
        
        // Initially stale (never loaded)
        assert!(cache.is_stale("availability", Duration::from_secs(60)));
        
        // Mark as updated
        cache.mark_updated("availability");
        
        // Should not be stale immediately
        assert!(!cache.is_stale("availability", Duration::from_secs(60)));
        
        // Should be stale with zero TTL
        assert!(cache.is_stale("availability", Duration::from_secs(0)));
    }
    
    #[test]
    fn test_time_range_serialization() {
        let time_range = TimeRange {
            start: "09:00".to_string(),
            end: "17:00".to_string(),
        };
        
        let json = serde_json::to_string(&time_range).unwrap();
        let deserialized: TimeRange = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.start, "09:00");
        assert_eq!(deserialized.end, "17:00");
    }
    
    #[test]
    fn test_loyalty_config_bonus_multipliers() {
        let mut bonus_multipliers = HashMap::new();
        bonus_multipliers.insert(1, Decimal::from(2));
        bonus_multipliers.insert(2, Decimal::from(3));
        
        let config = LoyaltyConfig {
            config_id: 1,
            points_per_dollar: Decimal::from(1),
            bonus_multipliers,
            updated_at: Utc::now(),
        };
        
        assert_eq!(config.bonus_multipliers.get(&1), Some(&Decimal::from(2)));
        assert_eq!(config.bonus_multipliers.get(&2), Some(&Decimal::from(3)));
        assert_eq!(config.bonus_multipliers.get(&3), None);
    }
}
