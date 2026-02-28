-- Create orders table for order management system
CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'confirmed', 'preparing', 'ready', 'completed', 'cancelled')),
    payment_status VARCHAR(50) NOT NULL DEFAULT 'unpaid' CHECK (payment_status IN ('unpaid', 'paid', 'refunded')),
    total_price DECIMAL(10, 2) NOT NULL CHECK (total_price >= 0),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create index for querying orders by user (for order history)
CREATE INDEX idx_orders_user_id ON orders(user_id);

-- Create index for querying orders by status (for filtering)
CREATE INDEX idx_orders_status ON orders(status);

-- Create index for sorting orders by creation time
CREATE INDEX idx_orders_created_at ON orders(created_at DESC);

-- Create composite index for user + status queries (common filter pattern)
CREATE INDEX idx_orders_user_status ON orders(user_id, status);
