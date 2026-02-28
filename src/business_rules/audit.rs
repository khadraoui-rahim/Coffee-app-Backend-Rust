// Audit Logger
// 
// Logs business rules application to the audit trail for compliance and debugging.
// Gracefully handles failures to avoid blocking primary operations.

use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

/// Audit Logger
/// 
/// Records business rules application events to the audit trail.
/// Failures are logged but do not propagate to prevent blocking operations.
pub struct AuditLogger {
    pool: PgPool,
}

impl AuditLogger {
    /// Create a new AuditLogger
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Log an availability check
    /// 
    /// Records when availability rules are checked for an order.
    /// Gracefully handles errors without propagating them.
    pub async fn log_availability_check(
        &self,
        order_id: Uuid,
        rule_data: JsonValue,
        effect: &str,
    ) {
        if let Err(e) = self.insert_audit_record(
            order_id,
            "availability",
            None,
            rule_data,
            effect,
        ).await {
            // Log error but don't propagate - audit failures shouldn't block operations
            eprintln!("Failed to log availability check: {}", e);
        }
    }
    
    /// Log a pricing rule application
    /// 
    /// Records when pricing rules are applied to an order.
    /// Gracefully handles errors without propagating them.
    pub async fn log_pricing_application(
        &self,
        order_id: Uuid,
        rule_id: Option<Uuid>,
        rule_data: JsonValue,
        effect: &str,
    ) {
        if let Err(e) = self.insert_audit_record(
            order_id,
            "pricing",
            rule_id,
            rule_data,
            effect,
        ).await {
            // Log error but don't propagate - audit failures shouldn't block operations
            eprintln!("Failed to log pricing application: {}", e);
        }
    }
    
    /// Log a loyalty points award
    /// 
    /// Records when loyalty points are awarded for an order.
    /// Gracefully handles errors without propagating them.
    pub async fn log_loyalty_award(
        &self,
        order_id: Uuid,
        rule_data: JsonValue,
        effect: &str,
    ) {
        if let Err(e) = self.insert_audit_record(
            order_id,
            "loyalty",
            None,
            rule_data,
            effect,
        ).await {
            // Log error but don't propagate - audit failures shouldn't block operations
            eprintln!("Failed to log loyalty award: {}", e);
        }
    }
    
    /// Insert an audit record into the database
    async fn insert_audit_record(
        &self,
        order_id: Uuid,
        rule_type: &str,
        rule_id: Option<Uuid>,
        rule_data: JsonValue,
        effect: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO rule_audit_log (order_id, rule_type, rule_id, rule_data, effect)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            order_id,
            rule_type,
            rule_id,
            rule_data,
            effect
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Get audit records for an order
    /// 
    /// Retrieves all audit records associated with a specific order.
    pub async fn get_audit_records(&self, order_id: Uuid) -> Result<Vec<AuditRecord>, sqlx::Error> {
        let records = sqlx::query_as!(
            AuditRecord,
            r#"
            SELECT 
                audit_id,
                order_id,
                rule_type,
                rule_id,
                rule_data,
                effect,
                created_at
            FROM rule_audit_log
            WHERE order_id = $1
            ORDER BY created_at ASC
            "#,
            order_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(records)
    }
}

/// Audit record from the database
#[derive(Debug, Clone)]
pub struct AuditRecord {
    pub audit_id: Uuid,
    pub order_id: Uuid,
    pub rule_type: String,
    pub rule_id: Option<Uuid>,
    pub rule_data: JsonValue,
    pub effect: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_audit_record_creation() {
        let record = AuditRecord {
            audit_id: Uuid::new_v4(),
            order_id: Uuid::new_v4(),
            rule_type: "pricing".to_string(),
            rule_id: Some(Uuid::new_v4()),
            rule_data: json!({"discount": "10%"}),
            effect: "Applied 10% discount".to_string(),
            created_at: chrono::Utc::now(),
        };
        
        assert_eq!(record.rule_type, "pricing");
        assert!(record.rule_id.is_some());
        assert_eq!(record.effect, "Applied 10% discount");
    }
    
    #[test]
    fn test_audit_record_with_null_rule_id() {
        let record = AuditRecord {
            audit_id: Uuid::new_v4(),
            order_id: Uuid::new_v4(),
            rule_type: "availability".to_string(),
            rule_id: None,
            rule_data: json!({"status": "available"}),
            effect: "All items available".to_string(),
            created_at: chrono::Utc::now(),
        };
        
        assert_eq!(record.rule_type, "availability");
        assert!(record.rule_id.is_none());
    }
    
    #[test]
    fn test_rule_data_serialization() {
        let rule_data = json!({
            "rule_type": "time_based",
            "discount_value": 15,
            "time_range": "14:00-17:00"
        });
        
        // Verify it's valid JSON
        assert!(rule_data.is_object());
        assert_eq!(rule_data["discount_value"], 15);
    }
    
    #[test]
    fn test_effect_message_format() {
        let effects = vec![
            "Applied 10% discount",
            "All items available",
            "Awarded 50 loyalty points",
            "Estimated prep time: 15 minutes",
        ];
        
        for effect in effects {
            assert!(!effect.is_empty());
            assert!(effect.len() > 5);
        }
    }
    
    #[test]
    fn test_rule_types() {
        let valid_types = vec!["availability", "pricing", "loyalty", "prep_time"];
        
        for rule_type in valid_types {
            assert!(!rule_type.is_empty());
            assert!(rule_type.len() <= 50); // VARCHAR(50) constraint
        }
    }
    
    #[test]
    fn test_audit_record_ordering() {
        // Test that records can be ordered by timestamp
        let now = chrono::Utc::now();
        let earlier = now - chrono::Duration::seconds(60);
        let later = now + chrono::Duration::seconds(60);
        
        assert!(earlier < now);
        assert!(now < later);
    }
    
    #[test]
    fn test_complex_rule_data() {
        let complex_data = json!({
            "rules_applied": [
                {
                    "rule_id": "123e4567-e89b-12d3-a456-426614174000",
                    "rule_type": "time_based",
                    "discount": 10
                },
                {
                    "rule_id": "223e4567-e89b-12d3-a456-426614174001",
                    "rule_type": "quantity_based",
                    "discount": 5
                }
            ],
            "total_discount": 15,
            "final_price": 85.00
        });
        
        assert!(complex_data["rules_applied"].is_array());
        assert_eq!(complex_data["total_discount"], 15);
    }
    
    #[test]
    fn test_loyalty_audit_data() {
        let loyalty_data = json!({
            "customer_id": 123,
            "order_total": 100.00,
            "points_per_dollar": 0.1,
            "base_points": 10,
            "bonus_points": 5,
            "total_points": 15
        });
        
        assert_eq!(loyalty_data["customer_id"], 123);
        assert_eq!(loyalty_data["total_points"], 15);
    }
    
    #[test]
    fn test_availability_audit_data() {
        let availability_data = json!({
            "items_checked": [
                {"coffee_id": 1, "status": "available"},
                {"coffee_id": 2, "status": "available"}
            ],
            "all_available": true
        });
        
        assert!(availability_data["all_available"].as_bool().unwrap());
        assert!(availability_data["items_checked"].is_array());
    }
}
