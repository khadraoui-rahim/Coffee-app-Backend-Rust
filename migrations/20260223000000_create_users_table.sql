-- Create users table for authentication
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Add unique constraint on email
ALTER TABLE users ADD CONSTRAINT users_email_unique UNIQUE (email);

-- Create case-insensitive index on email for efficient lookups
CREATE INDEX idx_users_email_lower ON users(LOWER(email));
