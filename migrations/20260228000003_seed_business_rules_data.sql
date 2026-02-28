-- Seed data for business rules system

-- Insert default loyalty configuration
INSERT INTO loyalty_config (config_id, points_per_dollar, bonus_multipliers)
VALUES (
    1,
    1.0,
    '{}'::jsonb
);

-- Insert coffee availability for all existing coffees (mark all as available)
-- This assumes coffees already exist in the database
INSERT INTO coffee_availability (coffee_id, status, reason, updated_at)
SELECT 
    id,
    'available',
    NULL,
    NOW()
FROM coffees
ON CONFLICT (coffee_id) DO NOTHING;

-- Insert default prep time configuration for all coffees
-- Base times vary by coffee complexity
INSERT INTO prep_time_config (coffee_id, base_minutes, per_additional_item, updated_at)
SELECT 
    id,
    CASE 
        WHEN name IN ('Espresso', 'Americano', 'Turkish Coffee') THEN 2
        WHEN name IN ('Cappuccino', 'Latte', 'Macchiato', 'Cortado') THEN 3
        WHEN name IN ('Flat White', 'Cold Brew', 'Iced Latte') THEN 4
        WHEN name IN ('Mocha', 'Affogato') THEN 5
        ELSE 3
    END,
    1,
    NOW()
FROM coffees
ON CONFLICT (coffee_id) DO NOTHING;

-- Insert a sample promotional pricing rule (10% off all orders over $10)
INSERT INTO pricing_rules (rule_type, priority, rule_config, is_active, valid_from, valid_until)
VALUES (
    'quantity_based',
    100,
    '{
        "min_quantity": 3,
        "discount_type": "percentage",
        "discount_value": 10.0,
        "description": "10% off when ordering 3 or more items"
    }'::jsonb,
    true,
    NOW(),
    NOW() + INTERVAL '1 year'
);

-- Insert a sample time-based pricing rule (happy hour: 15% off between 2-4 PM)
INSERT INTO pricing_rules (rule_type, priority, rule_config, is_active, valid_from, valid_until)
VALUES (
    'time_based',
    90,
    '{
        "time_ranges": [
            {"start": "14:00", "end": "16:00"}
        ],
        "discount_type": "percentage",
        "discount_value": 15.0,
        "description": "Happy Hour: 15% off between 2-4 PM"
    }'::jsonb,
    true,
    NOW(),
    NOW() + INTERVAL '1 year'
);
