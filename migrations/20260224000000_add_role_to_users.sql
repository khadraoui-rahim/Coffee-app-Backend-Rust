-- Add role column to users table for authorization
ALTER TABLE users 
ADD COLUMN role TEXT NOT NULL DEFAULT 'user';

-- Add check constraint to ensure valid roles
ALTER TABLE users 
ADD CONSTRAINT chk_user_role 
CHECK (role IN ('admin', 'user'));

-- Create index for role-based queries
CREATE INDEX idx_users_role ON users(role);
