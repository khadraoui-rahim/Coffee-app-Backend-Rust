// HTTP handlers for order endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::auth::middleware::AuthenticatedUser;
use crate::orders::{
    CreateOrderRequest, OrderError, OrderResponse, OrderStatus, PaymentStatus,
    UpdatePaymentRequest, UpdateStatusRequest,
};

/// Query parameters for order history
#[derive(Debug, Deserialize)]
pub struct OrderHistoryQuery {
    /// Optional status filter
    pub status: Option<OrderStatus>,
}

/// Handler for POST /api/orders
/// Creates a new order for the authenticated user
pub async fn create_order_handler(
    State(state): State<crate::AppState>,
    user: AuthenticatedUser,
    Json(request): Json<CreateOrderRequest>,
) -> Result<(StatusCode, Json<OrderResponse>), OrderError> {
    // Validate request
    request
        .validate()
        .map_err(|e| OrderError::ValidationError(e.to_string()))?;

    // Create order
    let order = state
        .order_service
        .create_order(user.user_id, request)
        .await?;

    // Convert Order to OrderResponse
    let items = state
        .order_items_repo
        .find_by_order_id(order.id)
        .await?;

    let response = OrderResponse {
        id: order.id,
        user_id: order.user_id,
        status: order.status,
        payment_status: order.payment_status,
        total_price: order.total_price,
        items: items.into_iter().map(|item| item.into()).collect(),
        created_at: order.created_at,
        updated_at: order.updated_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Handler for GET /api/orders
/// Retrieves order history for the authenticated user
pub async fn get_order_history_handler(
    State(state): State<crate::AppState>,
    user: AuthenticatedUser,
    Query(query): Query<OrderHistoryQuery>,
) -> Result<Json<Vec<OrderResponse>>, OrderError> {
    // Get user orders with optional status filter
    let orders = state
        .order_service
        .get_user_orders(user.user_id, query.status)
        .await?;

    Ok(Json(orders))
}

/// Handler for GET /api/orders/{order_id}
/// Retrieves a specific order by ID
pub async fn get_order_by_id_handler(
    State(state): State<crate::AppState>,
    user: AuthenticatedUser,
    Path(order_id): Path<Uuid>,
) -> Result<Json<OrderResponse>, OrderError> {
    // Get order by ID (authorization check is done in service layer)
    let order = state
        .order_service
        .get_order_by_id(order_id, user.user_id)
        .await?;

    Ok(Json(order))
}

/// Handler for PATCH /api/orders/{order_id}/status
/// Updates the status of an order (Admin/Staff only)
pub async fn update_order_status_handler(
    State(state): State<crate::AppState>,
    _user: AuthenticatedUser, // TODO: Add role check for admin/staff
    Path(order_id): Path<Uuid>,
    Json(request): Json<UpdateStatusRequest>,
) -> Result<Json<OrderResponse>, OrderError> {
    // Validate request
    request
        .validate()
        .map_err(|e| OrderError::ValidationError(e.to_string()))?;

    // Update order status
    let order = state
        .order_service
        .update_order_status(order_id, request.status)
        .await?;

    // Fetch order items to build response
    let items = state
        .order_items_repo
        .find_by_order_id(order.id)
        .await?;

    let response = OrderResponse {
        id: order.id,
        user_id: order.user_id,
        status: order.status,
        payment_status: order.payment_status,
        total_price: order.total_price,
        items: items.into_iter().map(|item| item.into()).collect(),
        created_at: order.created_at,
        updated_at: order.updated_at,
    };

    Ok(Json(response))
}

/// Handler for PATCH /api/orders/{order_id}/payment
/// Updates the payment status of an order (Admin/Staff only)
pub async fn update_payment_status_handler(
    State(state): State<crate::AppState>,
    _user: AuthenticatedUser, // TODO: Add role check for admin/staff
    Path(order_id): Path<Uuid>,
    Json(request): Json<UpdatePaymentRequest>,
) -> Result<Json<OrderResponse>, OrderError> {
    // Validate request
    request
        .validate()
        .map_err(|e| OrderError::ValidationError(e.to_string()))?;

    // Update payment status
    let order = state
        .order_service
        .update_payment_status(order_id, request.payment_status)
        .await?;

    // Fetch order items to build response
    let items = state
        .order_items_repo
        .find_by_order_id(order.id)
        .await?;

    let response = OrderResponse {
        id: order.id,
        user_id: order.user_id,
        status: order.status,
        payment_status: order.payment_status,
        total_price: order.total_price,
        items: items.into_iter().map(|item| item.into()).collect(),
        created_at: order.created_at,
        updated_at: order.updated_at,
    };

    Ok(Json(response))
}
