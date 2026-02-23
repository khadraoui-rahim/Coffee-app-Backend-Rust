use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use crate::error::ApiError;

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
    tracing::debug!("Creating database connection pool");
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await?;
    
    tracing::info!("Database connection pool created successfully");
    Ok(pool)
}

/// Check if a coffee with the given name already exists
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `name` - Coffee name to check for duplicates
///
/// # Returns
/// * `Result<bool, ApiError>` - True if duplicate exists, false otherwise
///
/// # Example
/// ```
/// if check_duplicate_coffee(&pool, "Espresso").await? {
///     return Err(ApiError::Conflict { message: "Coffee already exists".to_string() });
/// }
/// ```
pub async fn check_duplicate_coffee(
    pool: &PgPool,
    name: &str,
) -> Result<bool, ApiError> {
    tracing::debug!("Checking for duplicate coffee: {}", name);
    
    let exists: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM coffees WHERE name = $1)"
    )
    .bind(name)
    .fetch_one(pool)
    .await?;
    
    let is_duplicate = exists.unwrap_or(false);
    if is_duplicate {
        tracing::debug!("Duplicate coffee found: {}", name);
    }
    
    Ok(is_duplicate)
}

/// Check if a coffee with the given name already exists, excluding a specific ID
/// This is used for update operations to allow keeping the same name
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `name` - Coffee name to check for duplicates
/// * `exclude_id` - ID of the coffee being updated (to exclude from duplicate check)
///
/// # Returns
/// * `Result<bool, ApiError>` - True if duplicate exists (excluding the specified ID), false otherwise
///
/// # Example
/// ```
/// if check_duplicate_coffee_excluding_id(&pool, "Espresso", 5).await? {
///     return Err(ApiError::Conflict { message: "Coffee name already exists".to_string() });
/// }
/// ```
pub async fn check_duplicate_coffee_excluding_id(
    pool: &PgPool,
    name: &str,
    exclude_id: i32,
) -> Result<bool, ApiError> {
    let exists: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM coffees WHERE name = $1 AND id != $2)"
    )
    .bind(name)
    .bind(exclude_id)
    .fetch_one(pool)
    .await?;
    
    Ok(exists.unwrap_or(false))
}

/// Example of a transaction-based multi-step operation
/// This demonstrates how to use transactions for operations that modify multiple tables
/// or require multiple steps to complete atomically.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `coffee_id` - ID of the coffee to update
/// * `new_price` - New price to set
///
/// # Returns
/// * `Result<(), ApiError>` - Success or error
///
/// # Transaction Behavior
/// - All operations within the transaction are atomic
/// - If any operation fails, all changes are automatically rolled back
/// - The transaction is committed only when all operations succeed
/// - Using the ? operator automatically triggers rollback on error
///
/// # Example
/// ```
/// // This will either complete all steps or rollback everything
/// update_coffee_price_with_transaction(&pool, 1, 5.99).await?;
/// ```
pub async fn update_coffee_price_with_transaction(
    pool: &PgPool,
    coffee_id: i32,
    new_price: f64,
) -> Result<(), ApiError> {
    // Begin a new transaction
    // The transaction will automatically rollback if dropped without commit
    let mut tx = pool.begin().await?;
    
    // Step 1: Verify the coffee exists
    let exists: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM coffees WHERE id = $1)"
    )
    .bind(coffee_id)
    .fetch_one(&mut *tx)
    .await?;
    
    if !exists.unwrap_or(false) {
        // Transaction is automatically rolled back when tx is dropped
        return Err(ApiError::NotFound {
            resource: "Coffee".to_string(),
            id: coffee_id.to_string(),
        });
    }
    
    // Step 2: Update the coffee price
    sqlx::query("UPDATE coffees SET price = $1 WHERE id = $2")
        .bind(new_price)
        .bind(coffee_id)
        .execute(&mut *tx)
        .await?;
    
    // Step 3: Could add more operations here (e.g., logging, audit trail)
    // All operations are part of the same transaction
    
    // Commit the transaction - this makes all changes permanent
    // If commit fails, changes are rolled back
    tx.commit().await?;
    
    Ok(())
}
