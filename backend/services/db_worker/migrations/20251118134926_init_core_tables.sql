CREATE TABLE IF NOT EXISTS admins (
    id BIGSERIAL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    password TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    password TEXT NOT NULL,
    balance BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS events (
    id BIGSERIAL PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    category TEXT NOT NULL,
    status TEXT NOT NULL,
    resolved_at TEXT,
    winning_outcome_id BIGINT,
    created_by BIGINT NOT NULL REFERENCES admins(id)
);

CREATE TABLE IF NOT EXISTS outcomes (
    id BIGSERIAL PRIMARY KEY,
    event_id BIGINT NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    status TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_outcomes_event_id ON outcomes (event_id);

CREATE TABLE IF NOT EXISTS markets (
    id BIGSERIAL PRIMARY KEY,
    outcome_id BIGINT NOT NULL REFERENCES outcomes(id) ON DELETE CASCADE,
    side TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_markets_outcome_side ON markets (outcome_id, side);

CREATE TABLE IF NOT EXISTS positions (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    market_id BIGINT NOT NULL REFERENCES markets(id) ON DELETE CASCADE,
    quantity BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_positions_user_id ON positions (user_id);
CREATE INDEX IF NOT EXISTS idx_positions_market_id ON positions (market_id);
