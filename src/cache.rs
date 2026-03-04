use redis::aio::ConnectionManager;
use redis::{Client, AsyncCommands};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Cache miss")]
    Miss,
}

pub struct CacheService {
    manager: ConnectionManager,
    default_ttl: Duration,
}

impl CacheService {
    pub async fn new(redis_url: &str) -> Result<Self, CacheError> {
        let client = Client::open(redis_url)?;
        let manager = ConnectionManager::new(client).await?;
        Ok(Self {
            manager,
            default_ttl: Duration::from_secs(300), // 5 minutes default
        })
    }
    
    pub async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>, CacheError> {
        let mut conn = self.manager.clone();
        let value: Option<String> = conn.get(key).await?;
        
        match value {
            Some(v) => {
                let deserialized = serde_json::from_str(&v)?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }
    
    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<(), CacheError> {
        let mut conn = self.manager.clone();
        let serialized = serde_json::to_string(value)?;
        let ttl_secs = ttl.unwrap_or(self.default_ttl).as_secs();
        
        conn.set_ex::<_, _, ()>(key, serialized, ttl_secs).await?;
        Ok(())
    }
    
    pub async fn invalidate(&self, key: &str) -> Result<(), CacheError> {
        let mut conn = self.manager.clone();
        let _: () = conn.del(key).await?;
        Ok(())
    }
    
    pub async fn invalidate_pattern(&self, pattern: &str) -> Result<(), CacheError> {
        let mut conn = self.manager.clone();
        
        // Get all keys matching the pattern
        let keys: Vec<String> = conn.keys(pattern).await?;
        
        // Delete all matching keys
        if !keys.is_empty() {
            let _: () = conn.del(keys).await?;
        }
        
        Ok(())
    }
    
    pub async fn warm_cache(&self) -> Result<(), CacheError> {
        // This is a placeholder for cache warming logic
        // In a real implementation, this would preload critical data
        tracing::info!("Cache warming initiated");
        Ok(())
    }
}

pub struct CacheKey;

impl CacheKey {
    pub fn coffee_list() -> String {
        "coffee:list".to_string()
    }
    
    pub fn coffee_by_id(id: i32) -> String {
        format!("coffee:{}", id)
    }
    
    pub fn business_rules() -> String {
        "business_rules:*".to_string()
    }
    
    pub fn user_orders(user_id: i32) -> String {
        format!("user:{}:orders", user_id)
    }
    
    pub fn reviews_by_coffee(coffee_id: i32) -> String {
        format!("reviews:coffee:{}", coffee_id)
    }
}

// TTL constants
pub mod ttl {
    use std::time::Duration;
    
    pub const COFFEE_CACHE: Duration = Duration::from_secs(300); // 5 minutes
    pub const BUSINESS_RULES: Duration = Duration::from_secs(600); // 10 minutes
    pub const USER_SESSION: Duration = Duration::from_secs(900); // 15 minutes
    pub const USER_ORDERS: Duration = Duration::from_secs(120); // 2 minutes
    pub const REVIEWS: Duration = Duration::from_secs(180); // 3 minutes
}

