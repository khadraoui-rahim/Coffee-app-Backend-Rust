use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};

/// Cache entry with TTL support
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    expires_at: DateTime<Utc>,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl_seconds: i64) -> Self {
        Self {
            value,
            expires_at: Utc::now() + Duration::seconds(ttl_seconds),
        }
    }

    fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// In-memory cache manager with TTL support
#[derive(Clone)]
pub struct CacheManager<T: Clone> {
    cache: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    default_ttl_seconds: i64,
}

impl<T: Clone> CacheManager<T> {
    /// Create a new cache manager with default TTL of 5 minutes (300 seconds)
    pub fn new() -> Self {
        Self::with_ttl(300)
    }

    /// Create a new cache manager with custom TTL in seconds
    pub fn with_ttl(ttl_seconds: i64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl_seconds: ttl_seconds,
        }
    }

    /// Generate a cache key from endpoint and parameters
    /// Format: analytics:{endpoint}:{hash(params)}
    pub fn generate_key(endpoint: &str, params: &impl Hash) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        params.hash(&mut hasher);
        let hash = hasher.finish();
        format!("analytics:{}:{:x}", endpoint, hash)
    }

    /// Get a value from the cache
    /// Returns None if key doesn't exist or entry is expired
    pub fn get(&self, key: &str) -> Option<T> {
        let cache = self.cache.read().ok()?;
        
        if let Some(entry) = cache.get(key) {
            if !entry.is_expired() {
                return Some(entry.value.clone());
            }
        }
        
        None
    }

    /// Set a value in the cache with default TTL
    pub fn set(&self, key: String, value: T) {
        self.set_with_ttl(key, value, self.default_ttl_seconds);
    }

    /// Set a value in the cache with custom TTL
    pub fn set_with_ttl(&self, key: String, value: T, ttl_seconds: i64) {
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(key, CacheEntry::new(value, ttl_seconds));
        }
    }

    /// Invalidate (remove) a specific cache entry
    pub fn invalidate(&self, key: &str) -> bool {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(key).is_some()
        } else {
            false
        }
    }

    /// Invalidate all cache entries matching a pattern
    pub fn invalidate_pattern(&self, pattern: &str) -> usize {
        if let Ok(mut cache) = self.cache.write() {
            let keys_to_remove: Vec<String> = cache
                .keys()
                .filter(|k| k.contains(pattern))
                .cloned()
                .collect();
            
            let count = keys_to_remove.len();
            for key in keys_to_remove {
                cache.remove(&key);
            }
            count
        } else {
            0
        }
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// Remove expired entries from the cache
    pub fn cleanup_expired(&self) -> usize {
        if let Ok(mut cache) = self.cache.write() {
            let expired_keys: Vec<String> = cache
                .iter()
                .filter(|(_, entry)| entry.is_expired())
                .map(|(key, _)| key.clone())
                .collect();
            
            let count = expired_keys.len();
            for key in expired_keys {
                cache.remove(&key);
            }
            count
        } else {
            0
        }
    }

    /// Get the number of entries in the cache (including expired)
    pub fn size(&self) -> usize {
        self.cache.read().map(|c| c.len()).unwrap_or(0)
    }

    /// Get the number of non-expired entries in the cache
    pub fn active_size(&self) -> usize {
        if let Ok(cache) = self.cache.read() {
            cache.iter().filter(|(_, entry)| !entry.is_expired()).count()
        } else {
            0
        }
    }
}

impl<T: Clone> Default for CacheManager<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Cacheable query parameters for generating cache keys
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct CacheableParams {
    pub endpoint: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub period: Option<String>,
    pub limit: Option<i32>,
    pub coffee_id: Option<i32>,
}

impl CacheableParams {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            start_date: None,
            end_date: None,
            period: None,
            limit: None,
            coffee_id: None,
        }
    }

    pub fn with_dates(mut self, start: Option<String>, end: Option<String>) -> Self {
        self.start_date = start;
        self.end_date = end;
        self
    }

    pub fn with_period(mut self, period: Option<String>) -> Self {
        self.period = period;
        self
    }

    pub fn with_limit(mut self, limit: Option<i32>) -> Self {
        self.limit = limit;
        self
    }

    pub fn with_coffee_id(mut self, coffee_id: Option<i32>) -> Self {
        self.coffee_id = coffee_id;
        self
    }

    pub fn generate_key(&self) -> String {
        CacheManager::<()>::generate_key(&self.endpoint, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;

    #[test]
    fn test_cache_set_and_get() {
        let cache = CacheManager::<String>::new();
        let key = "test_key".to_string();
        let value = "test_value".to_string();

        cache.set(key.clone(), value.clone());
        
        let result = cache.get(&key);
        assert_eq!(result, Some(value));
    }

    #[test]
    fn test_cache_miss() {
        let cache = CacheManager::<String>::new();
        let result = cache.get("nonexistent_key");
        assert_eq!(result, None);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = CacheManager::<String>::with_ttl(1); // 1 second TTL
        let key = "test_key".to_string();
        let value = "test_value".to_string();

        cache.set(key.clone(), value.clone());
        
        // Should be available immediately
        assert_eq!(cache.get(&key), Some(value.clone()));
        
        // Wait for expiration
        thread::sleep(StdDuration::from_secs(2));
        
        // Should be expired now
        assert_eq!(cache.get(&key), None);
    }

    #[test]
    fn test_cache_invalidate() {
        let cache = CacheManager::<String>::new();
        let key = "test_key".to_string();
        let value = "test_value".to_string();

        cache.set(key.clone(), value.clone());
        assert_eq!(cache.get(&key), Some(value));
        
        let invalidated = cache.invalidate(&key);
        assert!(invalidated);
        assert_eq!(cache.get(&key), None);
    }

    #[test]
    fn test_cache_invalidate_nonexistent() {
        let cache = CacheManager::<String>::new();
        let invalidated = cache.invalidate("nonexistent_key");
        assert!(!invalidated);
    }

    #[test]
    fn test_cache_clear() {
        let cache = CacheManager::<String>::new();
        
        cache.set("key1".to_string(), "value1".to_string());
        cache.set("key2".to_string(), "value2".to_string());
        
        assert_eq!(cache.size(), 2);
        
        cache.clear();
        
        assert_eq!(cache.size(), 0);
        assert_eq!(cache.get("key1"), None);
        assert_eq!(cache.get("key2"), None);
    }

    #[test]
    fn test_cache_cleanup_expired() {
        let cache = CacheManager::<String>::with_ttl(1);
        
        cache.set("key1".to_string(), "value1".to_string());
        cache.set("key2".to_string(), "value2".to_string());
        
        assert_eq!(cache.size(), 2);
        
        // Wait for expiration
        thread::sleep(StdDuration::from_secs(2));
        
        let cleaned = cache.cleanup_expired();
        assert_eq!(cleaned, 2);
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_cache_size() {
        let cache = CacheManager::<String>::new();
        
        assert_eq!(cache.size(), 0);
        
        cache.set("key1".to_string(), "value1".to_string());
        assert_eq!(cache.size(), 1);
        
        cache.set("key2".to_string(), "value2".to_string());
        assert_eq!(cache.size(), 2);
    }

    #[test]
    fn test_cache_active_size() {
        let cache = CacheManager::<String>::with_ttl(1);
        
        cache.set("key1".to_string(), "value1".to_string());
        cache.set("key2".to_string(), "value2".to_string());
        
        assert_eq!(cache.active_size(), 2);
        
        // Wait for expiration
        thread::sleep(StdDuration::from_secs(2));
        
        assert_eq!(cache.active_size(), 0);
        assert_eq!(cache.size(), 2); // Still in cache, just expired
    }

    #[test]
    fn test_generate_key() {
        let params1 = CacheableParams::new("sales/total")
            .with_dates(Some("2024-01-01".to_string()), Some("2024-01-31".to_string()));
        
        let params2 = CacheableParams::new("sales/total")
            .with_dates(Some("2024-01-01".to_string()), Some("2024-01-31".to_string()));
        
        let params3 = CacheableParams::new("sales/total")
            .with_dates(Some("2024-02-01".to_string()), Some("2024-02-28".to_string()));
        
        let key1 = params1.generate_key();
        let key2 = params2.generate_key();
        let key3 = params3.generate_key();
        
        // Same parameters should generate same key
        assert_eq!(key1, key2);
        
        // Different parameters should generate different keys
        assert_ne!(key1, key3);
        
        // Keys should have the correct format
        assert!(key1.starts_with("analytics:sales/total:"));
    }

    #[test]
    fn test_cache_invalidate_pattern() {
        let cache = CacheManager::<String>::new();
        
        cache.set("analytics:sales/total:abc123".to_string(), "value1".to_string());
        cache.set("analytics:sales/by-period:def456".to_string(), "value2".to_string());
        cache.set("analytics:revenue/total:ghi789".to_string(), "value3".to_string());
        
        assert_eq!(cache.size(), 3);
        
        // Invalidate all sales-related cache entries
        let invalidated = cache.invalidate_pattern("sales");
        assert_eq!(invalidated, 2);
        assert_eq!(cache.size(), 1);
        
        // Revenue entry should still be there
        assert!(cache.get("analytics:revenue/total:ghi789").is_some());
    }

    #[test]
    fn test_cache_with_custom_ttl() {
        let cache = CacheManager::<String>::new();
        let key = "test_key".to_string();
        let value = "test_value".to_string();

        // Set with custom TTL of 1 second
        cache.set_with_ttl(key.clone(), value.clone(), 1);
        
        assert_eq!(cache.get(&key), Some(value));
        
        // Wait for expiration
        thread::sleep(StdDuration::from_secs(2));
        
        assert_eq!(cache.get(&key), None);
    }

    #[test]
    fn test_cacheable_params_builder() {
        let params = CacheableParams::new("sales/total")
            .with_dates(Some("2024-01-01".to_string()), Some("2024-01-31".to_string()))
            .with_period(Some("daily".to_string()))
            .with_limit(Some(10))
            .with_coffee_id(Some(5));
        
        assert_eq!(params.endpoint, "sales/total");
        assert_eq!(params.start_date, Some("2024-01-01".to_string()));
        assert_eq!(params.end_date, Some("2024-01-31".to_string()));
        assert_eq!(params.period, Some("daily".to_string()));
        assert_eq!(params.limit, Some(10));
        assert_eq!(params.coffee_id, Some(5));
    }

    #[test]
    fn test_cache_consistency_within_ttl() {
        // Property 27: Cache consistency within TTL
        let cache = CacheManager::<String>::with_ttl(10);
        let key = "test_key".to_string();
        let value = "test_value".to_string();

        cache.set(key.clone(), value.clone());
        
        // Multiple gets within TTL should return the same value
        for _ in 0..5 {
            assert_eq!(cache.get(&key), Some(value.clone()));
            thread::sleep(StdDuration::from_millis(100));
        }
    }

    #[test]
    fn test_cache_thread_safety() {
        let cache = CacheManager::<String>::new();
        let cache_clone = cache.clone();
        
        // Write from one thread
        let handle = thread::spawn(move || {
            cache_clone.set("key1".to_string(), "value1".to_string());
        });
        
        handle.join().unwrap();
        
        // Read from main thread
        assert_eq!(cache.get("key1"), Some("value1".to_string()));
    }
}
