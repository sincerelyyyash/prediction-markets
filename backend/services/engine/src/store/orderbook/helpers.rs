use crate::store::market::MarketStore;
use crate::types::market_types::MarketSide;
use crate::types::orderbook_types::{Order, OrderSide};

pub fn normalize_order(order: &mut Order, market_store: &MarketStore) -> Result<u64, String> {
    let Some(market) = market_store.get_market(order.market_id) else {
        return Err("Market not found".into());
    };

    if order.price > 100 {
        return Err("Price must be between 0 and 100".into());
    }

    if let Some(side) = &market.side {
        match side {
            MarketSide::No => {
                if let Some(paired_id) = market.paired_market_id {
                    order.market_id = paired_id;
                    order.price = 100 - order.price;
                    order.side = match order.side {
                        OrderSide::Bid => OrderSide::Ask,
                        OrderSide::Ask => OrderSide::Bid,
                    };
                }
            }
            MarketSide::Yes => {}
        }
    }

    Ok(order.market_id)
}

pub fn denormalize_price(market_id: u64, canonical_price: u64, market_store: &MarketStore) -> u64 {
    let Some(market) = market_store.get_market(market_id) else {
        return canonical_price;
    };

    if let Some(side) = &market.side {
        match side {
            MarketSide::No => 100 - canonical_price,
            MarketSide::Yes => canonical_price,
        }
    } else {
        canonical_price
    }
}
