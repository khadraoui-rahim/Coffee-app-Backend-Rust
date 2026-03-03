// Analytics data access layer
// Contains database queries for orders and reviews analytics

mod orders_repository;
mod reviews_repository;

pub use orders_repository::OrdersAnalyticsRepository;
pub use reviews_repository::ReviewsAnalyticsRepository;

#[cfg(test)]
mod tests;
