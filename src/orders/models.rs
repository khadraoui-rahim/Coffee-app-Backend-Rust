use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Order status enum representing the lifecycle of an order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Preparing,
    Ready,
    Completed,
    Cancelled,
}

impl OrderStatus {
    /// Convert status to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::Pending => "pending",
            OrderStatus::Confirmed => "confirmed",
            OrderStatus::Preparing => "preparing",
            OrderStatus::Ready => "ready",
            OrderStatus::Completed => "completed",
            OrderStatus::Cancelled => "cancelled",
        }
    }
    
    /// Parse status from string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(OrderStatus::Pending),
            "confirmed" => Ok(OrderStatus::Confirmed),
            "preparing" => Ok(OrderStatus::Preparing),
            "ready" => Ok(OrderStatus::Ready),
            "completed" => Ok(OrderStatus::Completed),
            "cancelled" => Ok(OrderStatus::Cancelled),
            _ => Err(format!("Invalid order status: {}", s)),
        }
    }
}

impl Default for OrderStatus {
    fn default() -> Self {
        OrderStatus::Pending
    }
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Payment status enum representing the payment state of an order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PaymentStatus {
    Unpaid,
    Paid,
    Refunded,
}

impl PaymentStatus {
    /// Convert payment status to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentStatus::Unpaid => "unpaid",
            PaymentStatus::Paid => "paid",
            PaymentStatus::Refunded => "refunded",
        }
    }
    
    /// Parse payment status from string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "unpaid" => Ok(PaymentStatus::Unpaid),
            "paid" => Ok(PaymentStatus::Paid),
            "refunded" => Ok(PaymentStatus::Refunded),
            _ => Err(format!("Invalid payment status: {}", s)),
        }
    }
}

impl Default for PaymentStatus {
    fn default() -> Self {
        PaymentStatus::Unpaid
    }
}

impl std::fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Domain model representing an order in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: Uuid,
    pub user_id: i32,
    pub status: OrderStatus,
    pub payment_status: PaymentStatus,
    pub total_price: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Domain model representing an item within an order
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrderItem {
    pub id: i32,
    pub order_id: Uuid,
    pub coffee_item_id: i32,
    pub quantity: i32,
    pub price_snapshot: Decimal,
    pub subtotal: Decimal,
}

/// Request DTO for creating an order item
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct OrderItemRequest {
    pub coffee_item_id: i32,
    #[validate(range(min = 1, message = "Quantity must be at least 1"))]
    pub quantity: i32,
}

/// Request DTO for creating a new order
#[derive(Debug, Deserialize, Validate)]
pub struct CreateOrderRequest {
    #[validate(length(min = 1, message = "Order must contain at least one item"))]
    pub items: Vec<OrderItemRequest>,
}

/// Request DTO for updating order status
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateStatusRequest {
    pub status: OrderStatus,
}

/// Request DTO for updating payment status
#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePaymentRequest {
    pub payment_status: PaymentStatus,
}

/// Response DTO for order with items
#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub id: Uuid,
    pub user_id: i32,
    pub status: OrderStatus,
    pub payment_status: PaymentStatus,
    pub total_price: Decimal,
    pub items: Vec<OrderItemResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response DTO for order item
#[derive(Debug, Serialize)]
pub struct OrderItemResponse {
    pub id: i32,
    pub coffee_item_id: i32,
    pub quantity: i32,
    pub price_snapshot: Decimal,
    pub subtotal: Decimal,
}

impl From<OrderItem> for OrderItemResponse {
    fn from(item: OrderItem) -> Self {
        Self {
            id: item.id,
            coffee_item_id: item.coffee_item_id,
            quantity: item.quantity,
            price_snapshot: item.price_snapshot,
            subtotal: item.subtotal,
        }
    }
}
