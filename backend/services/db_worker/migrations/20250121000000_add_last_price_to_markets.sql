-- Add last_price column to markets table
ALTER TABLE markets ADD COLUMN IF NOT EXISTS last_price BIGINT;

-- Backfill last_price from the most recent trade for each market
UPDATE markets m
SET last_price = (
    SELECT price
    FROM trades t
    WHERE t.market_id = m.id
    ORDER BY t.executed_at DESC
    LIMIT 1
)
WHERE EXISTS (
    SELECT 1
    FROM trades t
    WHERE t.market_id = m.id
);

-- Add index on last_price for queries
CREATE INDEX IF NOT EXISTS idx_markets_last_price ON markets (last_price);

