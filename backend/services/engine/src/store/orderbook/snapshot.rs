use std::collections::HashMap;

use crate::store::market::MarketStore;
use crate::store::orderbook::helpers::denormalize_price;
use crate::types::orderbook_types::{Level, OrderbookData, OrderbookSnapshot};

pub fn build_orderbook_snapshot(
    market_id: u64,
    alias_map: &HashMap<u64, u64>,
    orderbooks: &HashMap<u64, OrderbookData>,
    market_store: &MarketStore,
) -> Result<OrderbookSnapshot, String> {
    let canonical_id = alias_map.get(&market_id).copied().unwrap_or(market_id);
    let book = orderbooks
        .get(&canonical_id)
        .ok_or_else(|| "Market not found".to_string())?;

    let bids: Vec<Level> = book
        .bids
        .iter()
        .rev()
        .map(|(price, quantity)| Level {
            price: denormalize_price(market_id, *price, market_store),
            quantity: *quantity,
        })
        .collect();

    let asks: Vec<Level> = book
        .asks
        .iter()
        .map(|(price, quantity)| Level {
            price: denormalize_price(market_id, *price, market_store),
            quantity: *quantity,
        })
        .collect();

    let last_price = book
        .last_price
        .map(|p| denormalize_price(market_id, p, market_store));

    Ok(OrderbookSnapshot {
        market_id,
        bids,
        asks,
        last_price,
    })
}
