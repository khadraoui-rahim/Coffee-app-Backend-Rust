use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::Coffee;
use crate::orders::{Order, OrderItem, OrderStatus, PaymentStatus};
use crate::orders::error::OrderError;

/// Repository for coffee item operations
#[derive(Clone)]
pub struct CoffeeRepository {
    pool: PgPool,
}

impl CoffeeRepository {
    /// Create a new CoffeeRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find a coffee item by ID
    pub async fn find_by_id(&self, id: i32) -> Result<Option<Coffee>, OrderError> {
        let coffee = sqlx::query_as::<_, Coffee>(
            "SELECT id, image_url, name, coffee_type, price, rating FROM coffees WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(coffee)
    }

    /// Find multiple coffee items by IDs
    pub async fn find_by_ids(&self, ids: &[i32]) -> Result<Vec<Coffee>, OrderError> {
        let coffees = sqlx::query_as::<_, Coffee>(
            "SELECT id, image_url, name, coffee_type, price, rating FROM coffees WHERE id = ANY($1)"
        )
        .bind(ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(coffees)
    }
}

/// Repository for order operations
#[derive(Clone)]
pub struct OrdersRepository {
    pool: PgPool,
}

impl OrdersRepository {
    /// Create a new OrdersRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new order with items in a transaction
    pub async fn create(
        &self,
        user_id: i32,
        status: OrderStatus,
        payment_status: PaymentStatus,
        total_price: Decimal,
        items: Vec<(i32, i32, Decimal, Decimal)>, // (coffee_item_id, quantity, price_snapshot, subtotal)
    ) -> Result<Order, OrderError> {
        let mut tx = self.pool.begin().await?;

        // Insert order
        let order = sqlx::query_as::<_, Order>(
            r#"
            INSERT INTO orders (user_id, status, payment_status, total_price)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, status, payment_status, total_price, created_at, updated_at
            "#
        )
        .bind(user_id)
        .bind(status)
        .bind(payment_status)
        .bind(total_price)
        .fetch_one(&mut *tx)
        .await?;

        // Insert order items
        for (coffee_item_id, quantity, price_snapshot, subtotal) in items {
            sqlx::query(
                r#"
                INSERT INTO order_items (order_id, coffee_item_id, quantity, price_snapshot, subtotal)
                VALUES ($1, $2, $3, $4, $5)
                "#
            )
            .bind(order.id)
            .bind(coffee_item_id)
            .bind(quantity)
            .bind(price_snapshot)
            .bind(subtotal)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(order)
    }

    /// Find an order by ID
    pub async fn find_by_id(&self, order_id: Uuid) -> Result<Option<Order>, OrderError> {
        let order = sqlx::query_as::<_, Order>(
            r#"
            SELECT id, user_id, status, payment_status, total_price, created_at, updated_at
            FROM orders
            WHERE id = $1
            "#
        )
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(order)
    }

    /// Find orders by user ID with optional status filter
    pub async fn find_by_user_id(
        &self,
        user_id: i32,
        status: Option<OrderStatus>,
    ) -> Result<Vec<Order>, OrderError> {
        let orders = match status {
            Some(status_filter) => {
                sqlx::query_as::<_, Order>(
                    r#"
                    SELECT id, user_id, status, payment_status, total_price, created_at, updated_at
                    FROM orders
                    WHERE user_id = $1 AND status = $2
                    ORDER BY created_at DESC
                    "#
                )
                .bind(user_id)
                .bind(status_filter)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, Order>(
                    r#"
                    SELECT id, user_id, status, payment_status, total_price, created_at, updated_at
                    FROM orders
                    WHERE user_id = $1
                    ORDER BY created_at DESC
                    "#
                )
                .bind(user_id)
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(orders)
    }

    /// Update order status
    pub async fn update_status(
        &self,
        order_id: Uuid,
        new_status: OrderStatus,
    ) -> Result<Order, OrderError> {
        let order = sqlx::query_as::<_, Order>(
            r#"
            UPDATE orders
            SET status = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, user_id, status, payment_status, total_price, created_at, updated_at
            "#
        )
        .bind(new_status)
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(OrderError::NotFound)?;

        Ok(order)
    }

    /// Update payment status
    pub async fn update_payment_status(
        &self,
        order_id: Uuid,
        new_payment_status: PaymentStatus,
    ) -> Result<Order, OrderError> {
        let order = sqlx::query_as::<_, Order>(
            r#"
            UPDATE orders
            SET payment_status = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, user_id, status, payment_status, total_price, created_at, updated_at
            "#
        )
        .bind(new_payment_status)
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(OrderError::NotFound)?;

        Ok(order)
    }
}

/// Repository for order items operations
#[derive(Clone)]
pub struct OrderItemsRepository {
    pool: PgPool,
}

impl OrderItemsRepository {
    /// Create a new OrderItemsRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find all items for a given order
    pub async fn find_by_order_id(&self, order_id: Uuid) -> Result<Vec<OrderItem>, OrderError> {
        let items = sqlx::query_as::<_, OrderItem>(
            r#"
            SELECT id, order_id, coffee_item_id, quantity, price_snapshot, subtotal
            FROM order_items
            WHERE order_id = $1
            ORDER BY id
            "#
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Integration tests with actual database operations
    // would require testcontainers and are beyond the scope of unit tests.
    // The repository methods will be tested through service layer integration tests.
}
