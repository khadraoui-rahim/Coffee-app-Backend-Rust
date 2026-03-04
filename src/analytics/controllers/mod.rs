// Analytics API controllers
// Contains handlers for sales, revenue, popular coffees, and ratings endpoints

pub mod sales_controller;
pub mod popular_coffees_controller;
pub mod revenue_controller;
pub mod rating_insights_controller;

pub use sales_controller::SalesStatisticsController;
pub use popular_coffees_controller::PopularCoffeesController;
pub use revenue_controller::RevenueReportsController;
pub use rating_insights_controller::RatingInsightsController;
