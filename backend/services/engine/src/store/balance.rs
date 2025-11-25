use chrono::Utc;
use std::collections::HashMap;

use crate::services::db_event_publisher::publish_db_event;
use crate::types::db_event_types::{BalanceUpdatedEvent, DbEvent, PositionUpdatedEvent};
use crate::types::orderbook_types::{Order, OrderSide};
use crate::types::user_types::User;

pub async fn reserve_balance(order: &Order, users: &mut HashMap<u64, User>) -> Result<(), String> {
    match order.side {
        OrderSide::Bid => {
            let total_cost = (order.original_qty as i64) * (order.price as i64);
            update_balance(users, order.user_id, -total_cost)
        }
        OrderSide::Ask => {
            if !check_position_sufficient(users, order.user_id, order.market_id, order.original_qty)
            {
                return Err("Insufficient positions quantity".into());
            }
            update_position(
                users,
                order.user_id,
                order.market_id,
                -(order.original_qty as i64),
            )
        }
    }
}

pub async fn return_reserved_balance(
    order: &Order,
    users: &mut HashMap<u64, User>,
) -> Result<(), String> {
    match order.side {
        OrderSide::Bid => {
            let reserved = (order.remaining_qty as i64) * (order.price as i64);
            update_balance(users, order.user_id, reserved)
        }
        OrderSide::Ask => update_position(
            users,
            order.user_id,
            order.market_id,
            order.remaining_qty as i64,
        ),
    }
}

pub async fn return_unused_reservation(
    order: &Order,
    users: &mut HashMap<u64, User>,
) -> Result<(), String> {
    match order.side {
        OrderSide::Bid => {
            let original_reservation = (order.original_qty as i64) * (order.price as i64);
            let used = ((order.original_qty - order.remaining_qty) as i64) * (order.price as i64);
            let unused = original_reservation - used;
            if unused > 0 {
                update_balance(users, order.user_id, unused)
            } else {
                Ok(())
            }
        }
        OrderSide::Ask => {
            let unused = order.remaining_qty as i64;
            if unused > 0 {
                update_position(users, order.user_id, order.market_id, unused)
            } else {
                Ok(())
            }
        }
    }
}

fn update_balance(users: &mut HashMap<u64, User>, user_id: u64, amount: i64) -> Result<(), String> {
    let Some(user) = users.get_mut(&user_id) else {
        return Err("User not found".into());
    };
    user.balance += amount;

    let balance = user.balance;
    let timestamp = Utc::now();
    tokio::spawn(async move {
        let _ = publish_db_event(DbEvent::BalanceUpdated(BalanceUpdatedEvent {
            user_id,
            balance,
            timestamp,
        }))
        .await;
    });

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
    let final_qty = if *current == 0 {
        user.positions.remove(&market_id);
        0
    } else {
        *current
    };

    let quantity = final_qty;
    let timestamp = Utc::now();
    tokio::spawn(async move {
        let _ = publish_db_event(DbEvent::PositionUpdated(PositionUpdatedEvent {
            user_id,
            market_id,
            quantity,
            timestamp,
        }))
        .await;
    });

    Ok(())
}

fn check_position_sufficient(
    users: &HashMap<u64, User>,
    user_id: u64,
    market_id: u64,
    required_qty: u64,
) -> bool {
    users
        .get(&user_id)
        .and_then(|u| u.positions.get(&market_id))
        .copied()
        .unwrap_or(0)
        >= required_qty
}
