use crate::types::orderbook_types::{Order, OrderSide};
use crate::store::user::UserStore;

pub async fn reserve_balance( order: &Order, user_store: &UserStore) -> Result<(), String> {
    match order.side {
        OrderSide::Bid => {
            let total_cost = (order.original_qty as i64) * (order.price as i64);
            user_store.update_balance(order.user_id, -total_cost).await
            .map_err(|e| format!("Insufficient balance: {}", e))
        }
        OrderSide::Ask => {
            let has_sufficient = user_store.check_position_sufficient(
                order.user_id, 
                order.market_id, 
    order.original_qty
        ).await.map_err(|e| format!("Failed to check positions {}",e))?;

        if !has_sufficient {
            return Err("Insufficient positions quantity".into());
        }

        user_store.update_position(order.user_id, order.market_id, -(order.original_qty as i64)).await
        .map_err(|e| format!("insufficient position: {}", e))
        }
    }
}

pub async fn return_reserved_balance(order: &Order, user_store: &UserStore) -> Result<(), String> {
    match order.side {
        OrderSide::Bid => {
            let reserved = (order.remaining_qty as i64) * (order.price as i64);
            user_store.update_balance(order.user_id, reserved).await
            .map_err(|e| format!("Failed to return balance: {}", e))
        }
        OrderSide::Ask => {
            user_store.update_position(order.user_id, order.market_id, order.remaining_qty as i64).await
            .map_err(|e| format!("Failed to return position: {}", e))
        }
    }
}

pub async fn return_unused_reservation(
    order: &Order,
    user_store: &UserStore
) -> Result<(), String> {
    match order.side {
        OrderSide::Bid => {
            let original_reservation = (order.original_qty as i64) * (order.price as i64);
            let used = ((order.original_qty - order.remaining_qty) as i64) * (order.price as i64);
            let unused = original_reservation - used;
            if unused > 0 {
                user_store.update_balance(order.user_id, unused).await
                .map_err(|e| format!("Failed to return unused balance: {}", e))
            } else {
                 Ok(())
            }
        }
        OrderSide::Ask => {
            let unused = order.remaining_qty as i64;
            if unused > 0 {
                user_store.update_position(order.user_id, order.market_id, unused).await
                .map_err(|e|format!("Failed to return unused position: {}", e))
            } else {
                Ok(())
            }
        }
    }
}