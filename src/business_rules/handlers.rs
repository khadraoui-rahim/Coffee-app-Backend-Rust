// HTTP handlers for business rules management endpoints

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::auth::middleware::AuthenticatedUser;
use crate::business_rules::{
    AvailabilityStatus, BusinessRulesError, CombinationStrategy, DiscountType, LoyaltyConfig,
    PricingRuleType, TimeRange,
};

/// Request DTO for updating coffee availability
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAvailabilityRequest {
    pub status: AvailabilityStatus,
    pub available_from: Option<chrono::NaiveTime>,
    pub available_until: Option<chrono::NaiveTime>,
}

/// Response DTO for coffee availability
#[derive(Debug, Serialize)]
pub struct AvailabilityResponse {
    pub coffee_id: i32,
    pub status: AvailabilityStatus,
    pub available_from: Option<chrono::NaiveTime>,
    pub available_until: Option<chrono::NaiveTime>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Request DTO for creating a pricing rule
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePricingRuleRequest {
    pub rule_type: PricingRuleType,
    pub description: String,
    pub discount_type: DiscountType,
    pub discount_value: rust_decimal::Decimal,
    pub priority: i32,
    pub valid_from: Option<chrono::DateTime<chrono::Utc>>,
    pub valid_until: Option<chrono::DateTime<chrono::Utc>>,
    pub coffee_ids: Option<Vec<i32>>,
    pub time_ranges: Option<Vec<TimeRange>>,
    pub min_quantity: Option<i32>,
}

/// Response DTO for pricing rule
#[derive(Debug, Serialize)]
pub struct PricingRuleResponse {
    pub id: i32,
    pub rule_type: PricingRuleType,
    pub description: String,
    pub discount_type: DiscountType,
    pub discount_value: rust_decimal::Decimal,
    pub priority: i32,
    pub is_active: bool,
    pub valid_from: Option<chrono::DateTime<chrono::Utc>>,
    pub valid_until: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Request DTO for updating loyalty configuration
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateLoyaltyConfigRequest {
    #[validate(range(min = 0.0))]
    pub points_per_dollar: f64,
    pub bonus_multipliers: Option<serde_json::Value>,
}

/// Request DTO for updating prep time configuration
#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePrepTimeRequest {
    #[validate(range(min = 1))]
    pub base_minutes: i32,
    #[validate(range(min = 0))]
    pub per_additional_item: i32,
}

/// Handler for POST /api/business-rules/availability
/// Updates coffee availability status (Admin only)
pub async fn update_availability_handler(
    State(state): State<crate::AppState>,
    _user: AuthenticatedUser,
    Json(request): Json<UpdateAvailabilityRequest>,
) -> Result<(StatusCode, Json<AvailabilityResponse>), BusinessRulesError> {
    request.validate()?;
    
    // Implementation will be added when integrating with the business rules engine
    todo!("Implement availability update")
}

/// Handler for GET /api/business-rules/availability/:coffee_id
/// Gets availability status for a specific coffee
pub async fn get_availability_handler(
    State(state): State<crate::AppState>,
    Path(coffee_id): Path<i32>,
) -> Result<Json<AvailabilityResponse>, BusinessRulesError> {
    // Implementation will be added when integrating with the business rules engine
    todo!("Implement get availability")
}

/// Handler for POST /api/business-rules/pricing
/// Creates a new pricing rule (Admin only)
pub async fn create_pricing_rule_handler(
    State(state): State<crate::AppState>,
    _user: AuthenticatedUser,
    Json(request): Json<CreatePricingRuleRequest>,
) -> Result<(StatusCode, Json<PricingRuleResponse>), BusinessRulesError> {
    request.validate()?;
    
    // Implementation will be added when integrating with the business rules engine
    todo!("Implement create pricing rule")
}

/// Handler for PUT /api/business-rules/pricing/:rule_id
/// Updates an existing pricing rule (Admin only)
pub async fn update_pricing_rule_handler(
    State(state): State<crate::AppState>,
    _user: AuthenticatedUser,
    Path(rule_id): Path<i32>,
    Json(request): Json<CreatePricingRuleRequest>,
) -> Result<Json<PricingRuleResponse>, BusinessRulesError> {
    request.validate()?;
    
    // Implementation will be added when integrating with the business rules engine
    todo!("Implement update pricing rule")
}

/// Handler for DELETE /api/business-rules/pricing/:rule_id
/// Deactivates a pricing rule (Admin only)
pub async fn delete_pricing_rule_handler(
    State(state): State<crate::AppState>,
    _user: AuthenticatedUser,
    Path(rule_id): Path<i32>,
) -> Result<StatusCode, BusinessRulesError> {
    // Implementation will be added when integrating with the business rules engine
    todo!("Implement delete pricing rule")
}

/// Handler for GET /api/business-rules/pricing
/// Lists all active pricing rules
pub async fn list_pricing_rules_handler(
    State(state): State<crate::AppState>,
) -> Result<Json<Vec<PricingRuleResponse>>, BusinessRulesError> {
    // Implementation will be added when integrating with the business rules engine
    todo!("Implement list pricing rules")
}

/// Handler for PUT /api/business-rules/loyalty-config
/// Updates loyalty configuration (Admin only)
pub async fn update_loyalty_config_handler(
    State(state): State<crate::AppState>,
    _user: AuthenticatedUser,
    Json(request): Json<UpdateLoyaltyConfigRequest>,
) -> Result<Json<LoyaltyConfig>, BusinessRulesError> {
    request.validate()?;
    
    // Implementation will be added when integrating with the business rules engine
    todo!("Implement update loyalty config")
}

/// Handler for GET /api/business-rules/loyalty-config
/// Gets current loyalty configuration
pub async fn get_loyalty_config_handler(
    State(state): State<crate::AppState>,
) -> Result<Json<LoyaltyConfig>, BusinessRulesError> {
    // Implementation will be added when integrating with the business rules engine
    todo!("Implement get loyalty config")
}

/// Handler for PUT /api/business-rules/prep-time/:coffee_id
/// Updates prep time configuration for a coffee (Admin only)
pub async fn update_prep_time_handler(
    State(state): State<crate::AppState>,
    _user: AuthenticatedUser,
    Path(coffee_id): Path<i32>,
    Json(request): Json<UpdatePrepTimeRequest>,
) -> Result<StatusCode, BusinessRulesError> {
    request.validate()?;
    
    // Implementation will be added when integrating with the business rules engine
    todo!("Implement update prep time")
}

/// Handler for GET /api/business-rules/metrics
/// Gets performance metrics for the business rules system
pub async fn get_metrics_handler(
    State(state): State<crate::AppState>,
) -> Result<Json<serde_json::Value>, BusinessRulesError> {
    let summary = state.business_rules_engine.metrics().summary();
    
    Ok(Json(serde_json::json!({
        "cache": {
            "hit_rate": format!("{:.1}%", summary.cache_hit_rate * 100.0),
            "hits": summary.cache_hits,
            "misses": summary.cache_misses,
        },
        "availability": {
            "checks": summary.availability_checks,
            "avg_time_ms": format!("{:.2}", summary.avg_availability_time_ms),
            "slow_operations": summary.slow_availability_checks,
        },
        "pricing": {
            "calculations": summary.pricing_calculations,
            "avg_time_ms": format!("{:.2}", summary.avg_pricing_time_ms),
            "slow_operations": summary.slow_pricing_calculations,
        },
        "prep_time": {
            "estimates": summary.prep_time_estimates,
            "avg_time_ms": format!("{:.2}", summary.avg_prep_time_ms),
            "slow_operations": summary.slow_prep_time_estimates,
        },
        "loyalty": {
            "calculations": summary.loyalty_calculations,
            "avg_time_ms": format!("{:.2}", summary.avg_loyalty_time_ms),
            "slow_operations": summary.slow_loyalty_calculations,
        },
    })))
}
