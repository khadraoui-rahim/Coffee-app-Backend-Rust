-- Simplify coffees table to match frontend model
-- Drop old table and recreate with simplified schema
DROP TABLE IF EXISTS coffees CASCADE;

CREATE TABLE coffees (
    id SERIAL PRIMARY KEY,
    image_url TEXT NOT NULL,
    name VARCHAR(255) NOT NULL,
    coffee_type VARCHAR(255) NOT NULL,
    price DOUBLE PRECISION NOT NULL CHECK (price > 0),
    rating DOUBLE PRECISION NOT NULL CHECK (rating >= 0.0 AND rating <= 5.0)
);

-- Create index for frequently queried fields
CREATE INDEX idx_coffees_name ON coffees(name);

-- Insert sample data matching frontend
INSERT INTO coffees (image_url, name, coffee_type, price, rating) VALUES
('https://images.unsplash.com/photo-1594146971821-373461fd5cd8?q=80&w=687&auto=format&fit=crop&ixlib=rb-4.1.0&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D', 'Caffe Mocha', 'Deep Foam', 4.53, 4.8),
('https://images.unsplash.com/photo-1587466959442-d4d155afc586?q=80&w=1074&auto=format&fit=crop&ixlib=rb-4.1.0&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D', 'Flat White', 'Espresso', 3.89, 4.5),
('https://images.unsplash.com/photo-1622843404078-a09315f46ae3?q=80&w=1074&auto=format&fit=crop&ixlib=rb-4.1.0&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D', 'Cappuccino', 'With Chocolate', 4.20, 4.7),
('https://images.unsplash.com/photo-1557238574-aca2834bae47?q=80&w=735&auto=format&fit=crop&ixlib=rb-4.1.0&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D', 'Caramel Macchiato', 'With Oat Milk', 4.15, 4.9),
('https://images.unsplash.com/photo-1585594467309-b726b6ba2fb5?q=80&w=1170&auto=format&fit=crop&ixlib=rb-4.1.0&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D', 'Americano', 'Double Shot', 3.50, 4.6);
