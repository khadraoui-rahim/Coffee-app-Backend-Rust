-- Create reviews table for ratings and reviews system
CREATE TABLE reviews (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    coffee_id INTEGER NOT NULL REFERENCES coffees(id) ON DELETE CASCADE,
    rating SMALLINT NOT NULL CHECK (rating >= 1 AND rating <= 5),
    comment TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    CONSTRAINT reviews_user_coffee_unique UNIQUE(user_id, coffee_id)
);

-- Create index for querying reviews by coffee (for listing all reviews of a coffee)
CREATE INDEX idx_reviews_coffee_id ON reviews(coffee_id);

-- Create composite index for duplicate detection (user + coffee combination)
CREATE INDEX idx_reviews_user_coffee ON reviews(user_id, coffee_id);
