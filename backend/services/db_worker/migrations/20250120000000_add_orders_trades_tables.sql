CREATE TABLE IF NOT EXISTS orders (
    id BIGSERIAL PRIMARY KEY,
    order_id BIGINT NOT NULL UNIQUE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    market_id BIGINT NOT NULL REFERENCES markets(id) ON DELETE CASCADE,
    side TEXT NOT NULL,
    price BIGINT NOT NULL,
    original_qty BIGINT NOT NULL,
    remaining_qty BIGINT NOT NULL,
    filled_qty BIGINT NOT NULL DEFAULT 0,
    status TEXT NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT NOW(),
    cancelled_at TIMESTAMP WITHOUT TIME ZONE,
    filled_at TIMESTAMP WITHOUT TIME ZONE
);

CREATE INDEX IF NOT EXISTS idx_orders_user_id ON orders (user_id);
CREATE INDEX IF NOT EXISTS idx_orders_market_id ON orders (market_id);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders (status);
CREATE INDEX IF NOT EXISTS idx_orders_created_at ON orders (created_at);

CREATE TABLE IF NOT EXISTS trades (
    id BIGSERIAL PRIMARY KEY,
    trade_id TEXT NOT NULL UNIQUE,
    market_id BIGINT NOT NULL REFERENCES markets(id) ON DELETE CASCADE,
    taker_order_id BIGINT NOT NULL,
    maker_order_id BIGINT NOT NULL,
    taker_user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    maker_user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    price BIGINT NOT NULL,
    quantity BIGINT NOT NULL,
    taker_side TEXT NOT NULL,
    executed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_trades_market_id ON trades (market_id);
CREATE INDEX IF NOT EXISTS idx_trades_taker_user_id ON trades (taker_user_id);
CREATE INDEX IF NOT EXISTS idx_trades_maker_user_id ON trades (maker_user_id);
CREATE INDEX IF NOT EXISTS idx_trades_executed_at ON trades (executed_at);
CREATE INDEX IF NOT EXISTS idx_trades_taker_order_id ON trades (taker_order_id);
CREATE INDEX IF NOT EXISTS idx_trades_maker_order_id ON trades (maker_order_id);

