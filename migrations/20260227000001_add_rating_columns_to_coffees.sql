-- Add average_rating and review_count columns to coffees table
-- These columns will be maintained by the rating calculator

-- Add average_rating column (NUMERIC(3,2) allows values like 4.25, range 0.00 to 5.00)
ALTER TABLE coffees ADD COLUMN average_rating NUMERIC(3,2);

-- Add review_count column to track total number of reviews
ALTER TABLE coffees ADD COLUMN review_count INTEGER NOT NULL DEFAULT 0;

-- Add check constraint to ensure average_rating is in valid range when not null
ALTER TABLE coffees ADD CONSTRAINT coffees_average_rating_check 
    CHECK (average_rating IS NULL OR (average_rating >= 0.0 AND average_rating <= 5.0));
