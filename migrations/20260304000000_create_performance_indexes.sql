-- Create indexes for performance optimization
-- These indexes improve query performance for filtering, sorting, and pagination

-- Coffees table indexes
-- Composite index for filtering by type and price
CREATE INDEX IF NOT EXISTS idx_coffees_type_price ON coffees(coffee_type, price);

-- Index for sorting by rating (descending order for "highest rated" queries)
CREATE INDEX IF NOT EXISTS idx_coffees_rating_desc ON coffees(rating DESC);

-- Orders table indexes
-- Composite index for user-specific order queries with pagination (most recent first)
CREATE INDEX IF NOT EXISTS idx_orders_user_created_desc ON orders(user_id, created_at DESC);

-- Partial index for active orders only (excludes completed/cancelled orders)
-- This improves performance for queries that only care about in-progress orders
CREATE INDEX IF NOT EXISTS idx_orders_active ON orders(user_id, status) 
WHERE status IN ('pending', 'confirmed', 'preparing', 'ready');

-- Reviews table indexes
-- Composite index for coffee-specific review queries with pagination (most recent first)
CREATE INDEX IF NOT EXISTS idx_reviews_coffee_created_desc ON reviews(coffee_id, created_at DESC);

-- Comments explaining index usage:
-- 
-- idx_coffees_type_price: Used for queries like:
--   - Filter coffees by type and sort by price
--   - Find coffees within a price range for a specific type
--   - Coffee menu queries with type filtering
--
-- idx_coffees_rating_desc: Used for queries like:
--   - Get highest rated coffees
--   - Sort coffee list by rating (descending)
--   - "Top rated" coffee queries
--
-- idx_orders_user_created_desc: Used for queries like:
--   - Get user's order history (most recent first)
--   - Paginate through user orders
--   - User-specific order queries with time-based sorting
--
-- idx_orders_active: Used for queries like:
--   - Get user's active/in-progress orders
--   - Dashboard showing current orders
--   - Queries that exclude completed/cancelled orders
--   - Partial index reduces index size and improves write performance
--
-- idx_reviews_coffee_created_desc: Used for queries like:
--   - Get reviews for a specific coffee (most recent first)
--   - Paginate through coffee reviews
--   - Coffee detail page review listing
