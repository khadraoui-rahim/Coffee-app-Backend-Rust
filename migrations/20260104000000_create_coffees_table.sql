-- Create coffees table
CREATE TABLE coffees (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    coffee_type VARCHAR(255) NOT NULL,
    price INTEGER NOT NULL CHECK (price > 0),
    rating DECIMAL(2,1) NOT NULL CHECK (rating >= 0.0 AND rating <= 5.0),
    temperature VARCHAR(10) NOT NULL CHECK (temperature IN ('hot', 'cold', 'both')),
    description TEXT NOT NULL,
    size VARCHAR(50) NOT NULL,
    liked BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create indexes for frequently queried fields
CREATE INDEX idx_coffees_name ON coffees(name);
CREATE INDEX idx_coffees_temperature ON coffees(temperature);

-- Create function to auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger to automatically update updated_at on row updates
CREATE TRIGGER update_coffees_updated_at 
    BEFORE UPDATE ON coffees
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();
