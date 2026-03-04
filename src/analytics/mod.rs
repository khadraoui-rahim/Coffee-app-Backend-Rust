pub mod types;
pub mod controllers;
pub mod services;
pub mod repositories;
pub mod middleware;
pub mod utils;
pub mod routes;
pub mod error;
pub mod validation;
pub mod formatting;

pub use types::*;
pub use middleware::AnalyticsAuthMiddleware;
pub use routes::create_analytics_router;
pub use error::AnalyticsError;
pub use validation::AnalyticsValidator;
pub use formatting::ResponseFormatter;

#[cfg(test)]
mod tests;
