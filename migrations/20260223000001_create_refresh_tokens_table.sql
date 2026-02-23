-- Create refresh_tokens table for token management
CREATE TABLE refresh_tokens (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    token_hash VARCHAR(64) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Add foreign key constraint to users table
ALTER TABLE refresh_tokens 
    ADD CONSTRAINT fk_refresh_tokens_user_id 
    FOREIGN KEY (user_id) 
    REFERENCES users(id) 
    ON DELETE CASCADE;

-- Create index on user_id for efficient lookups by user
CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);

-- Create index on expires_at for efficient cleanup of expired tokens
CREATE INDEX idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);

-- Create index on token_hash for efficient token verification
CREATE INDEX idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);
