-- Create order_items table for storing items in each order
CREATE TABLE order_items (
    id SERIAL PRIMARY KEY,
    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    coffee_item_id INTEGER NOT NULL REFERENCES coffees(id) ON DELETE RESTRICT,
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    price_snapshot DECIMAL(10, 2) NOT NULL CHECK (price_snapshot > 0),
    subtotal DECIMAL(10, 2) NOT NULL CHECK (subtotal >= 0)
);

-- Create index for querying items by order (for retrieving order details)
CREATE INDEX idx_order_items_order_id ON order_items(order_id);

-- Create index for querying items by coffee (for analytics)
CREATE INDEX idx_order_items_coffee_id ON order_items(coffee_item_id);
