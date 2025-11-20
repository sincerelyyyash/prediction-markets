use std::collections::HashMap;

use crate::store::market::MarketStore;
use crate::types::market_types::MarketStatus;
use crate::types::orderbook_types::{Order, OrderSide, OrderbookData};
use crate::types::user_types::User;

pub async fn match_order(
    order: &mut Order,
    book: &mut OrderbookData,
    users: &mut HashMap<u64, User>,
    market_store: &MarketStore,
) -> Result<(), String> {
    let Some(market) = market_store.get_market(order.market_id) else {
        return Err("Market not found".into());
    };

    if market.status != MarketStatus::Active {
        return Err("Market not active to trade".into());
    };

    match order.side {
        OrderSide::Bid => match_bid_against_asks(order, book, users).await,
        OrderSide::Ask => match_ask_against_bids(order, book, users).await,
    }
}

async fn match_bid_against_asks(
    order: &mut Order,
    book: &mut OrderbookData,
    users: &mut HashMap<u64, User>,
) -> Result<(), String> {
    while order.remaining_qty > 0 {
        let Some((&ask_price, _)) = book.asks.first_key_value() else {
            break;
        };

        if order.price < ask_price {
            break;
        }

        let Some(order_ids) = book.ask_queue.get_mut(&ask_price) else {
            book.asks.remove(&ask_price);
            continue;
        };

        while let Some(&maker_order_id) = order_ids.first() {
            let Some(maker_order) = book.orders.get_mut(&maker_order_id) else {
                order_ids.remove(0);
                continue;
            };

            let fill_qty = order.remaining_qty.min(maker_order.remaining_qty);
            let fill_price = ask_price;

            order.remaining_qty -= fill_qty;
            maker_order.remaining_qty -= fill_qty;

            *book.asks.get_mut(&ask_price).unwrap() -= fill_qty;

            let price_diff = (order.price as i64) - (fill_price as i64);
            let refund = price_diff * (fill_qty as i64);
            update_balance(users, order.user_id, refund)?;

            update_position(users, order.user_id, order.market_id, fill_qty as i64)?;

            let maker_revenue = (fill_price as i64) * (fill_qty as i64);
            update_balance(users, maker_order.user_id, maker_revenue)?;

            update_position(
                users,
                maker_order.user_id,
                maker_order.market_id,
                -(fill_qty as i64),
            )?;

            if maker_order.remaining_qty == 0 {
                book.orders.remove(&maker_order_id);
                order_ids.remove(0);

                if order_ids.is_empty() {
                    book.ask_queue.remove(&ask_price);
                    book.asks.remove(&ask_price);
                    break;
                }
            } else {
                break;
            }
            if order.remaining_qty == 0 {
                break;
            }
        }
    }
    Ok(())
}

async fn match_ask_against_bids(
    order: &mut Order,
    book: &mut OrderbookData,
    users: &mut HashMap<u64, User>,
) -> Result<(), String> {
    while order.remaining_qty > 0 {
        let Some((&bid_price, _)) = book.bids.last_key_value() else {
            break;
        };

        if order.price > bid_price {
            break;
        }

        let Some(order_ids) = book.bid_queue.get_mut(&bid_price) else {
            book.bids.remove(&bid_price);
            continue;
        };

        while let Some(&maker_order_id) = order_ids.first() {
            let Some(maker_order) = book.orders.get_mut(&maker_order_id) else {
                order_ids.remove(0);
                continue;
            };

            let fill_qty = order.remaining_qty.min(maker_order.remaining_qty);
            let fill_price = bid_price;

            order.remaining_qty -= fill_qty;
            maker_order.remaining_qty -= fill_qty;

            *book.bids.get_mut(&bid_price).unwrap() -= fill_qty;

            let payment = (fill_price as i64) * (fill_qty as i64);
            update_balance(users, order.user_id, payment)?;

            update_position(users, order.user_id, order.market_id, -(fill_qty as i64))?;

            let maker_price_diff = (maker_order.price as i64) - (fill_price as i64);
            let maker_refund = maker_price_diff * (fill_qty as i64);
            update_balance(users, maker_order.user_id, maker_refund)?;

            update_position(
                users,
                maker_order.user_id,
                maker_order.market_id,
                fill_qty as i64,
            )?;

            if maker_order.remaining_qty == 0 {
                book.orders.remove(&maker_order_id);
                order_ids.remove(0);

                if order_ids.is_empty() {
                    book.bid_queue.remove(&bid_price);
                    book.bids.remove(&bid_price);
                    break;
                }
            } else {
                break;
            }

            if order.remaining_qty == 0 {
                break;
            }
        }
    }
    Ok(())
}

fn update_balance(users: &mut HashMap<u64, User>, user_id: u64, amount: i64) -> Result<(), String> {
    let Some(user) = users.get_mut(&user_id) else {
        return Err("User not found".into());
    };
    user.balance += amount;
    Ok(())
}

fn update_position(
    users: &mut HashMap<u64, User>,
    user_id: u64,
    market_id: u64,
    amount: i64,
) -> Result<(), String> {
    let Some(user) = users.get_mut(&user_id) else {
        return Err("User not found".into());
    };
    let current = user.positions.entry(market_id).or_insert(0);
    if amount < 0 && (*current as i64) < -amount {
        return Err("Insufficient position".into());
    }
    *current = ((*current as i64) + amount) as u64;
    if *current == 0 {
        user.positions.remove(&market_id);
    }
    Ok(())
}
