-- Create business rules system tables

-- Coffee availability status
CREATE TABLE coffee_availability (
    coffee_id INTEGER PRIMARY KEY REFERENCES coffees(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL CHECK (status IN ('available', 'out_of_stock', 'seasonal', 'discontinued')),
    reason TEXT,
    available_from TIMESTAMPTZ,
    available_until TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Pricing rules configuration
CREATE TABLE pricing_rules (
    rule_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_type VARCHAR(50) NOT NULL CHECK (rule_type IN ('time_based', 'quantity_based', 'promotional')),
    priority INTEGER NOT NULL DEFAULT 0,
    rule_config JSONB NOT NULL,
    coffee_ids INTEGER[],
    is_active BOOLEAN NOT NULL DEFAULT true,
    valid_from TIMESTAMPTZ NOT NULL,
    valid_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Prep time configuration
CREATE TABLE prep_time_config (
    coffee_id INTEGER PRIMARY KEY REFERENCES coffees(id) ON DELETE CASCADE,
    base_minutes INTEGER NOT NULL CHECK (base_minutes > 0),
    per_additional_item INTEGER NOT NULL DEFAULT 0 CHECK (per_additional_item >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Loyalty configuration (singleton table)
CREATE TABLE loyalty_config (
    config_id INTEGER PRIMARY KEY DEFAULT 1,
    points_per_dollar DECIMAL(10, 4) NOT NULL CHECK (points_per_dollar >= 0),
    bonus_multipliers JSONB NOT NULL DEFAULT '{}',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT single_row CHECK (config_id = 1)
);

-- Customer loyalty balances
CREATE TABLE customer_loyalty (
    customer_id INTEGER PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    points_balance INTEGER NOT NULL DEFAULT 0 CHECK (points_balance >= 0),
    lifetime_points INTEGER NOT NULL DEFAULT 0 CHECK (lifetime_points >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Rule audit trail
CREATE TABLE rule_audit_log (
    audit_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    rule_type VARCHAR(50) NOT NULL,
    rule_id UUID,
    rule_data JSONB NOT NULL,
    effect TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_rule_audit_order ON rule_audit_log(order_id);
CREATE INDEX idx_rule_audit_created ON rule_audit_log(created_at);
CREATE INDEX idx_pricing_rules_active ON pricing_rules(is_active, priority) WHERE is_active = true;

-- Extend orders table with business rules columns
ALTER TABLE orders ADD COLUMN base_price DECIMAL(10, 2);
ALTER TABLE orders ADD COLUMN final_price DECIMAL(10, 2);
ALTER TABLE orders ADD COLUMN estimated_prep_minutes INTEGER;
ALTER TABLE orders ADD COLUMN loyalty_points_awarded INTEGER DEFAULT 0;
