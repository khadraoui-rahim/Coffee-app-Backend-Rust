-- Create indexes for analytics queries optimization
-- These indexes improve performance for sales, revenue, and rating analytics

-- Orders table indexes for analytics
-- Composite index for time-based queries with status filtering
CREATE INDEX IF NOT EXISTS idx_orders_created_status ON orders(created_at, status);

-- Composite index for coffee-specific order analytics
CREATE INDEX IF NOT EXISTS idx_order_items_coffee_created ON order_items(coffee_item_id, order_id);

-- Reviews table indexes for analytics
-- Composite index for coffee-specific rating queries
CREATE INDEX IF NOT EXISTS idx_reviews_coffee_rating ON reviews(coffee_id, rating);

-- Composite index for time-based review analytics
CREATE INDEX IF NOT EXISTS idx_reviews_created_coffee ON reviews(created_at, coffee_id);

-- Additional index for time-based review queries without coffee filter
CREATE INDEX IF NOT EXISTS idx_reviews_created_at ON reviews(created_at);

-- Comments explaining index usage:
-- 
-- idx_orders_created_status: Used for queries like:
--   - Count orders by date range and status
--   - Aggregate sales by time period (completed orders only)
--   - Revenue calculations by date range
--
-- idx_order_items_coffee_created: Used for queries like:
--   - Revenue by coffee type
--   - Most ordered coffees
--   - Coffee-specific sales analytics
--
-- idx_reviews_coffee_rating: Used for queries like:
--   - Average rating by coffee
--   - Rating distribution by coffee
--   - Highest rated coffees
--
-- idx_reviews_created_coffee: Used for queries like:
--   - Review trends over time
--   - Coffee-specific rating trends
--   - Time-based rating analytics
--
-- idx_reviews_created_at: Used for queries like:
--   - Overall review trends
--   - Time-based rating statistics without coffee filter
