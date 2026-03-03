// Analytics business logic services
// Contains sales aggregation, revenue calculation, popular coffees, and rating analysis services

mod sales_aggregation_service;
mod revenue_calculation_service;
mod popular_coffees_service;
mod trend_calculation_service;
mod rating_analysis_service;

pub use sales_aggregation_service::SalesAggregationService;
pub use revenue_calculation_service::RevenueCalculationService;
pub use popular_coffees_service::PopularCoffeesService;
pub use trend_calculation_service::TrendCalculationService;
pub use rating_analysis_service::RatingAnalysisService;

#[cfg(test)]
mod tests;
