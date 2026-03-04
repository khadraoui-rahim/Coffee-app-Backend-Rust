// Analytics API routes
// Configures routing for all analytics endpoints

use axum::{
    middleware,
    routing::get,
    Router,
};
use std::sync::Arc;

use crate::analytics::controllers::{PopularCoffeesController, RatingInsightsController, RevenueReportsController, SalesStatisticsController};
use crate::analytics::middleware::logging_middleware;
use crate::auth::middleware::RequireRole;

/// Create the analytics router with all endpoints
/// Base path: /api/v1/admin/analytics
pub fn create_analytics_router(
    sales_controller: Arc<SalesStatisticsController>,
    popular_coffees_controller: Arc<PopularCoffeesController>,
    revenue_controller: Arc<RevenueReportsController>,
    rating_insights_controller: Arc<RatingInsightsController>,
) -> Router {
    // Sales statistics routes
    let sales_routes = Router::new()
        .route("/total", get(SalesStatisticsController::get_total_sales))
        .route("/by-period", get(SalesStatisticsController::get_sales_by_period))
        .route("/trends", get(SalesStatisticsController::get_sales_trends))
        .with_state(sales_controller);

    // Popular coffees routes
    let coffees_routes = Router::new()
        .route("/most-ordered", get(PopularCoffeesController::get_most_ordered))
        .route("/highest-rated", get(PopularCoffeesController::get_highest_rated))
        .route("/trending", get(PopularCoffeesController::get_trending))
        .with_state(popular_coffees_controller);

    // Revenue reports routes
    let revenue_routes = Router::new()
        .route("/by-period", get(RevenueReportsController::get_revenue_by_period))
        .route("/by-coffee", get(RevenueReportsController::get_revenue_by_coffee))
        .with_state(revenue_controller);

    // Rating insights routes
    let rating_routes = Router::new()
        .route("/average", get(RatingInsightsController::get_average_rating))
        .route("/distribution", get(RatingInsightsController::get_rating_distribution))
        .route("/trends", get(RatingInsightsController::get_rating_trends))
        .with_state(rating_insights_controller);

    // Combine all analytics routes with admin authentication and logging
    Router::new()
        .nest("/sales", sales_routes)
        .nest("/coffees", coffees_routes)
        .nest("/revenue", revenue_routes)
        .nest("/ratings", rating_routes)
        .layer(middleware::from_fn(logging_middleware))
        .layer(middleware::from_fn(|req, next| {
            RequireRole::admin().middleware(req, next)
        }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::{
        repositories::{OrdersAnalyticsRepository, ReviewsAnalyticsRepository},
        services::{PopularCoffeesService, RatingAnalysisService, RevenueCalculationService, SalesAggregationService, TrendCalculationService},
    };
    use sqlx::PgPool;

    #[test]
    fn test_router_creation() {
        let pool = PgPool::connect_lazy("postgresql://test").unwrap();
        
        // Sales controller
        let orders_repo = OrdersAnalyticsRepository::new(pool.clone());
        let sales_service = Arc::new(SalesAggregationService::new(orders_repo.clone()));
        let sales_controller = Arc::new(SalesStatisticsController::new(sales_service));
        
        // Popular coffees controller
        let reviews_repo = ReviewsAnalyticsRepository::new(pool.clone());
        let popular_service = Arc::new(PopularCoffeesService::new(
            orders_repo.clone(),
            reviews_repo.clone(),
        ));
        let trend_service = Arc::new(TrendCalculationService::new(orders_repo.clone()));
        let popular_coffees_controller = Arc::new(PopularCoffeesController::new(
            popular_service,
            trend_service,
        ));
        
        // Revenue controller
        let revenue_service = Arc::new(RevenueCalculationService::new(orders_repo));
        let revenue_controller = Arc::new(RevenueReportsController::new(revenue_service));
        
        // Rating insights controller
        let rating_service = Arc::new(RatingAnalysisService::new(reviews_repo));
        let rating_insights_controller = Arc::new(RatingInsightsController::new(rating_service));
        
        let router = create_analytics_router(
            sales_controller,
            popular_coffees_controller,
            revenue_controller,
            rating_insights_controller,
        );
        
        // Router should be created successfully
        assert!(std::mem::size_of_val(&router) > 0);
    }
}
