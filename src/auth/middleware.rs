// Authentication middleware for protected routes

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use crate::auth::error::AuthError;

/// Authenticated user extractor for protected routes
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: i32,
    pub email: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // TODO: Implement in Task 11.1
        todo!("Implement authentication middleware")
    }
}
