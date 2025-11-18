use crate::types::orderbook_types::{OrderbookData, Order, OrderSide};

pub fn remove_order_from_book(order_id: u64, order: &Order, book: &mut OrderbookData){
    let price = order.price;
    let qty = order.remaining_qty;

    let queue_map = match order.side {
        OrderSide::Ask => &mut book.ask_queue,
        OrderSide::Bid => &mut book.bid_queue,
    };

    if let Some(queue) = queue_map.get_mut(&price) {
        queue.retain(|&id| id != order_id);
        if queue.is_empty() {
            queue_map.remove(&price);
        }
    }

    let map = match order.side {
        OrderSide::Ask => &mut book.asks,
        OrderSide::Bid => &mut book.bids,
    };

    if let Some(level_qty) = map.get_mut(&price) {
        *level_qty -= qty;
        if *level_qty == 0 {
            map.remove(&price);
        }
    }

    book.orders.remove(&order_id);
}

pub fn add_order_to_book(order_id: u64, order: &Order, book: &mut OrderbookData) {
    book.orders.insert(order_id, order.clone());

    match order.side {
        OrderSide::Bid => {
            book.bid_queue.entry(order.price).or_default().push(order_id);
            *book.bids.entry(order.price).or_default() += order.remaining_qty;
        }
        OrderSide::Ask => {
            book.ask_queue.entry(order.price).or_default().push(order_id);
            *book.asks.entry(order.price).or_default() += order.remaining_qty;
        }
    }
}