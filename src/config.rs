use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Environment variable {0} not found")]
    MissingEnvVar(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
}

#[derive(Debug, Clone)]
pub struct ConnectionPoolConfig {
    pub min_connections: u32,
    pub max_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl ConnectionPoolConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let min_connections = std::env::var("DB_MIN_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u32>()
            .map_err(|e| ConfigError::ParseError(format!("DB_MIN_CONNECTIONS: {}", e)))?;
        
        let max_connections = std::env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "20".to_string())
            .parse::<u32>()
            .map_err(|e| ConfigError::ParseError(format!("DB_MAX_CONNECTIONS: {}", e)))?;
        
        let connect_timeout_secs = std::env::var("DB_CONNECT_TIMEOUT")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u64>()
            .map_err(|e| ConfigError::ParseError(format!("DB_CONNECT_TIMEOUT: {}", e)))?;
        
        let idle_timeout_secs = std::env::var("DB_IDLE_TIMEOUT")
            .unwrap_or_else(|_| "30".to_string())
            .parse::<u64>()
            .map_err(|e| ConfigError::ParseError(format!("DB_IDLE_TIMEOUT: {}", e)))?;
        
        let max_lifetime_secs = std::env::var("DB_MAX_LIFETIME")
            .unwrap_or_else(|_| "1800".to_string())
            .parse::<u64>()
            .map_err(|e| ConfigError::ParseError(format!("DB_MAX_LIFETIME: {}", e)))?;
        
        let config = Self {
            min_connections,
            max_connections,
            connect_timeout: Duration::from_secs(connect_timeout_secs),
            idle_timeout: Duration::from_secs(idle_timeout_secs),
            max_lifetime: Duration::from_secs(max_lifetime_secs),
        };
        
        config.validate()?;
        Ok(config)
    }
    
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.min_connections >= self.max_connections {
            return Err(ConfigError::InvalidConfig(
                format!(
                    "min_connections ({}) must be less than max_connections ({})",
                    self.min_connections, self.max_connections
                )
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub enabled: bool,
}

impl RedisConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        
        let enabled = std::env::var("CACHE_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .map_err(|e| ConfigError::ParseError(format!("CACHE_ENABLED: {}", e)))?;
        
        Ok(Self { url, enabled })
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    pub redis: RedisConfig,
    pub connection_pool: ConnectionPoolConfig,
}

impl PerformanceConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            redis: RedisConfig::from_env()?,
            connection_pool: ConnectionPoolConfig::from_env()?,
        })
    }
}
