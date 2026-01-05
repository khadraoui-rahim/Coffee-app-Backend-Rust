use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Type alias for the PostgreSQL connection pool
pub type DbPool = PgPool;

/// Creates and configures a PostgreSQL connection pool
///
/// # Arguments
/// * `database_url` - PostgreSQL connection string
///
/// # Returns
/// * `Result<DbPool>` - Configured connection pool or error
///
/// # Example
/// ```
/// let pool = create_pool("postgresql://user:pass@localhost/db").await?;
/// ```
pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await
}
