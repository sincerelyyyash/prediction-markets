-- Seed test data for testing endpoints
-- Note: Passwords are bcrypt hashes of "password123"
-- Generated using bcrypt with cost 4 for testing
-- Hash: $2b$04$GG3FX1cD9mJZr4wK6OP5dOgmpMqwQ1qrflBQDjUZhTy6gWJOqvg9G

-- Insert test admin
INSERT INTO admins (id, email, name, password) 
VALUES (
    1,
    'admin@example.com',
    'Test Admin',
    '$2b$04$GG3FX1cD9mJZr4wK6OP5dOgmpMqwQ1qrflBQDjUZhTy6gWJOqvg9G'
) ON CONFLICT (email) DO NOTHING;

-- Insert test user
INSERT INTO users (id, email, name, password, balance) 
VALUES (
    1,
    'test@example.com',
    'Test User',
    '$2b$04$GG3FX1cD9mJZr4wK6OP5dOgmpMqwQ1qrflBQDjUZhTy6gWJOqvg9G',
    10000
) ON CONFLICT (email) DO NOTHING;

-- Insert another test user for trading
INSERT INTO users (id, email, name, password, balance) 
VALUES (
    2,
    'trader@example.com',
    'Test Trader',
    '$2b$04$GG3FX1cD9mJZr4wK6OP5dOgmpMqwQ1qrflBQDjUZhTy6gWJOqvg9G',
    50000
) ON CONFLICT (email) DO NOTHING;

