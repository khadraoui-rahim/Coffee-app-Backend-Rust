-- Seed data for production coffee database
-- This file contains initial coffee items for the application

-- Clear existing data (use with caution in production!)
-- TRUNCATE TABLE coffees RESTART IDENTITY CASCADE;

-- Insert sample coffee items
INSERT INTO coffees (image_url, name, coffee_type, price, rating) VALUES
('https://images.unsplash.com/photo-1510591509098-f4fdc6d0ff04?w=400', 'Espresso', 'Single Shot', 3.50, 4.8),
('https://images.unsplash.com/photo-1572442388796-11668a67e53d?w=400', 'Cappuccino', 'With Chocolate', 4.50, 4.9),
('https://images.unsplash.com/photo-1461023058943-07fcbe16d735?w=400', 'Latte', 'With Milk', 4.00, 4.7),
('https://images.unsplash.com/photo-1485808191679-5f86510681a2?w=400', 'Americano', 'Black Coffee', 3.00, 4.5),
('https://images.unsplash.com/photo-1517487881594-2787fef5ebf7?w=400', 'Mocha', 'With Chocolate', 5.00, 4.9),
('https://images.unsplash.com/photo-1511920170033-f8396924c348?w=400', 'Macchiato', 'Espresso Macchiato', 3.75, 4.6),
('https://images.unsplash.com/photo-1514432324607-a09d9b4aefdd?w=400', 'Flat White', 'With Steamed Milk', 4.25, 4.8),
('https://images.unsplash.com/photo-1509042239860-f550ce710b93?w=400', 'Cortado', 'Espresso with Milk', 3.50, 4.7),
('https://images.unsplash.com/photo-1497935586351-b67a49e012bf?w=400', 'Affogato', 'With Ice Cream', 5.50, 5.0),
('https://images.unsplash.com/photo-1545665225-b23b99e4d45e?w=400', 'Cold Brew', 'Iced Coffee', 4.50, 4.6),
('https://images.unsplash.com/photo-1517487881594-2787fef5ebf7?w=400', 'Iced Latte', 'Cold Latte', 4.75, 4.8),
('https://images.unsplash.com/photo-1578374173705-0a5dc3c8d4e5?w=400', 'Turkish Coffee', 'Traditional', 3.25, 4.5);

-- Verify the data was inserted
SELECT COUNT(*) as total_coffees FROM coffees;
