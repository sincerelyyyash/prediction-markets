-- Add optional image metadata for every market
ALTER TABLE markets
ADD COLUMN IF NOT EXISTS img_url TEXT;

-- Create bookmark table to track user saved markets
CREATE TABLE IF NOT EXISTS user_market_bookmarks (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    market_id BIGINT NOT NULL REFERENCES markets(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, market_id)
);

CREATE INDEX IF NOT EXISTS idx_user_market_bookmarks_user ON user_market_bookmarks (user_id);
CREATE INDEX IF NOT EXISTS idx_user_market_bookmarks_market ON user_market_bookmarks (market_id);


