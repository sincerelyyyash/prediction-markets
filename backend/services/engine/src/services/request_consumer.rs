use redis_client::RedisManager;
use redis_client::{RedisRequest, RedisResponse};
use crate::store::orderbook::Orderbook;
use crate::types::orderbook_types::Order;
use crate::types::user_types::User;
use crate::types::request_types::*;
use crate::types::market_types::MarketMeta;
use log::{error, info, warn};
use fred::prelude::*;
use serde_json::{Value, json};
use std::collections::HashMap;

pub async fn start_request_consumer(orderbook: Orderbook) {
    let redis_manager = match RedisManager::global() {
        Some(rm) => rm,
        None => {
            error!("Redis manager not initialized, cannot start request consumer");
            return;
        }
    };

    let client = redis_manager.client();
    let stream_name = "server_requests";

    tokio::spawn(async move {
        info!("Starting request consumer for stream: {}", stream_name);
        let mut last_id = "0".to_string();

        loop {
            match read_stream_messages(&client, stream_name, &mut last_id).await {
                Ok(messages) => {
                    if !messages.is_empty() {
                        if let Err(e) = process_message(messages, &orderbook).await {
                            error!("Error processing message: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading from stream {}: {}", stream_name, e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });
}

async fn read_stream_messages(
    client: &RedisClient,
    stream: &str,
    last_id: &mut String,
) -> Result<Vec<(String, Vec<(String, Vec<(String, String)>)>)>, RedisError> {
    use std::collections::HashMap;
    
    let streams = vec![stream];
    let ids = vec![last_id.as_str()];
    
    let raw_result: RedisValue = match client
        .xread::<RedisValue, _, _>(Some(10), None, streams, ids)
        .await
    {
        Ok(result) => result,
        Err(e) => {
            let error_msg = e.to_string();
            error!("XREAD error: {}", error_msg);
            return Err(e);
        }
    };
    
    let mut result: Vec<(String, Vec<(String, HashMap<String, String>)>)> = Vec::new();
    
    if let RedisValue::Array(streams_array) = raw_result {
        for stream_entry in streams_array {
            if let RedisValue::Array(stream_data) = stream_entry {
                if stream_data.len() >= 2 {
                    let stream_name = stream_data[0].as_str().map(|s| s.to_string()).unwrap_or_else(|| String::new());
                    
                    if let RedisValue::Array(messages) = &stream_data[1] {
                        let mut stream_messages: Vec<(String, HashMap<String, String>)> = Vec::new();
                        
                        for message in messages {
                            if let RedisValue::Array(msg_data) = message {
                                if msg_data.len() >= 2 {
                                    let msg_id = msg_data[0].as_str().map(|s| s.to_string()).unwrap_or_else(|| String::new());
                                    
                                    if let RedisValue::Array(fields_array) = &msg_data[1] {
                                        let mut fields_map = HashMap::new();
                                        
                                        for i in (0..fields_array.len()).step_by(2) {
                                            if i + 1 < fields_array.len() {
                                                let key = fields_array[i].as_str().map(|s| s.to_string()).unwrap_or_else(|| String::new());
                                                let value = fields_array[i + 1].as_str().map(|s| s.to_string()).unwrap_or_else(|| String::new());
                                                fields_map.insert(key, value);
                                            }
                                        }
                                        
                                        if !msg_id.is_empty() && msg_id > *last_id {
                                            *last_id = msg_id.clone();
                                        }
                                        
                                        stream_messages.push((msg_id, fields_map));
                                    }
                                }
                            }
                        }
                        
                        if !stream_messages.is_empty() {
                            result.push((stream_name, stream_messages));
                        }
                    }
                }
            }
        }
    }
    
    // Convert to final format with Vec<(String, String)> for fields
    let mut formatted_result = Vec::new();
    for (stream_name, messages_vec) in result {
        let mut messages = Vec::new();
        for (msg_id, fields_map) in messages_vec {
            let fields: Vec<(String, String)> = fields_map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            messages.push((msg_id, fields));
        }
        if !messages.is_empty() {
            formatted_result.push((stream_name, messages));
        }
    }

    Ok(formatted_result)
}

async fn process_message(
    stream_data: Vec<(String, Vec<(String, Vec<(String, String)>)>)>,
    orderbook: &Orderbook,
) -> Result<(), String> {
    for (_stream_name, messages) in stream_data {
        for (_msg_id, fields) in messages {
            let mut request_id: Option<String> = None;
            let mut data_str: Option<String> = None;
            
            for (key, value) in &fields {
                if key == "request_id" {
                    request_id = Some(value.clone());
                } else if key == "data" {
                    data_str = Some(value.clone());
                }
            }
            
            let request_id = request_id.ok_or_else(|| "Missing request_id in message".to_string())?;
            let data_str = data_str.ok_or_else(|| "Missing data in message".to_string())?;

            let request: RedisRequest<Value> = serde_json::from_str(&data_str)
                .map_err(|e| format!("Failed to parse request: {}", e))?;

            info!("Processing request: action={}, request_id={}", request.action, request_id);

            let response = match request.action.as_str() {
                "place-order" => handle_place_order(request.data, orderbook).await,
                "cancel-order" => handle_cancel_order(request.data, orderbook).await,
                "modify-order" => handle_modify_order(request.data, orderbook).await,
                "get-open-orders" => handle_get_open_orders(request.data, orderbook).await,
                "get-order-status" => handle_get_order_status(request.data, orderbook).await,
                "get-order-history" => handle_get_order_history(request.data, orderbook).await,
                "create-user" => handle_create_user(request.data, orderbook).await,
                "get-balance" => handle_get_balance(request.data, orderbook).await,
                "onramp" => handle_onramp(request.data, orderbook).await,
                "get-positions" => handle_get_positions(request.data, orderbook).await,
                "get-position" => handle_get_position(request.data, orderbook).await,
                "get-portfolio" => handle_get_portfolio(request.data, orderbook).await,
                "split-order" => handle_split_order(request.data, orderbook).await,
                "merge-order" => handle_merge_order(request.data, orderbook).await,
                "init-event-markets" => handle_init_event_markets(request.data, orderbook).await,
                "close-event-markets" => handle_close_event_markets(request.data, orderbook).await,
                _ => {
                    warn!("Unknown action: {}", request.action);
                    Ok(RedisResponse::new(
                        400,
                        false,
                        format!("Unknown action: {}", request.action),
                        serde_json::json!(null),
                    ))
                }
            };

            match response {
                Ok(resp) => {
                    send_response(request_id, resp).await?;
                }
                Err(e) => {
                    error!("Error handling request {}: {}", request.action, e);
                    let error_response = RedisResponse::new(
                        500,
                        false,
                        format!("Error processing request: {}", e),
                        serde_json::json!(null),
                    );
                    send_response(request_id, error_response).await?;
                }
            }
        }
    }

    Ok(())
}

async fn handle_place_order(data: Value, orderbook: &Orderbook) -> Result<RedisResponse<Value>, String> {
    let req: PlaceOrderRequest = serde_json::from_value(data)
        .map_err(|e| format!("Invalid request data: {}", e))?;

    let price = match req.order_type {
        crate::types::orderbook_types::OrderType::Market => {
            match req.side {
                crate::types::orderbook_types::OrderSide::Bid => {
                    match orderbook.best_ask(req.market_id).await {
                        Ok(p) if p > 0 => p,
                        _ => {
                            return Ok(RedisResponse::new(
                                400,
                                false,
                                "No available ask price for market order".to_string(),
                                serde_json::json!(null),
                            ));
                        }
                    }
                }
                crate::types::orderbook_types::OrderSide::Ask => {
                    match orderbook.best_bid(req.market_id).await {
                        Ok(p) if p > 0 => p,
                        _ => {
                            return Ok(RedisResponse::new(
                                400,
                                false,
                                "No available bid price for market order".to_string(),
                                serde_json::json!(null),
                            ));
                        }
                    }
                }
            }
        }
        crate::types::orderbook_types::OrderType::Limit => {
            req.price.ok_or_else(|| "Price required for limit orders".to_string())?
        }
    };

    let order = Order {
        order_id: None,
        market_id: req.market_id,
        user_id: req.user_id,
        price,
        original_qty: req.original_qty,
        remaining_qty: req.remaining_qty,
        side: req.side,
        order_type: req.order_type,
    };

    match orderbook.place_order(order).await {
        Ok(result_order) => {
            let order_json = serde_json::to_value(&result_order)
                .map_err(|e| format!("Failed to serialize order: {}", e))?;
            Ok(RedisResponse::new(
                200,
                true,
                "Order placed successfully",
                order_json,
            ))
        }
        Err(e) => {
            Ok(RedisResponse::new(
                400,
                false,
                format!("Failed to place order: {}", e),
                serde_json::json!(null),
            ))
        }
    }
}

async fn handle_cancel_order(data: Value, orderbook: &Orderbook) -> Result<RedisResponse<Value>, String> {
    let req: CancelOrderRequest = serde_json::from_value(data)
        .map_err(|e| format!("Invalid request data: {}", e))?;

    match orderbook.cancel_order(req.market_id, req.order_id).await {
        Ok(result_order) => {
            let order_json = serde_json::to_value(&result_order)
                .map_err(|e| format!("Failed to serialize order: {}", e))?;
            Ok(RedisResponse::new(
                200,
                true,
                "Order cancelled successfully",
                order_json,
            ))
        }
        Err(e) => {
            Ok(RedisResponse::new(
                400,
                false,
                format!("Failed to cancel order: {}", e),
                serde_json::json!(null),
            ))
        }
    }
}

async fn handle_modify_order(data: Value, orderbook: &Orderbook) -> Result<RedisResponse<Value>, String> {
    let req: ModifyOrderRequest = serde_json::from_value(data)
        .map_err(|e| format!("Invalid request data: {}", e))?;
    
    let existing_order = match orderbook.get_order_status(req.order_id).await {
        Ok(order) => order,
        Err(_) => {
            return Ok(RedisResponse::new(
                404,
                false,
                "Order not found",
                serde_json::json!(null),
            ));
        }
    };

    let mut updated_order = existing_order.clone();
    
    if let Some(price) = req.price {
        updated_order.price = price;
    }
    if let Some(qty) = req.original_qty {
        updated_order.original_qty = qty;
        updated_order.remaining_qty = qty;
    }

    match orderbook.modify_order(updated_order).await {
        Ok(result_order) => {
            let order_json = serde_json::to_value(&result_order)
                .map_err(|e| format!("Failed to serialize order: {}", e))?;
            Ok(RedisResponse::new(
                200,
                true,
                "Order modified successfully",
                order_json,
            ))
        }
        Err(e) => {
            Ok(RedisResponse::new(
                400,
                false,
                format!("Failed to modify order: {}", e),
                serde_json::json!(null),
            ))
        }
    }
}

async fn handle_get_open_orders(data: Value, orderbook: &Orderbook) -> Result<RedisResponse<Value>, String> {
    let req: GetOpenOrdersRequest = serde_json::from_value(data)
        .map_err(|e| format!("Invalid request data: {}", e))?;

    match orderbook.get_user_open_orders(req.user_id).await {
        Ok(orders) => {
            let orders_json = serde_json::to_value(&orders)
                .map_err(|e| format!("Failed to serialize orders: {}", e))?;
            Ok(RedisResponse::new(
                200,
                true,
                "Open orders retrieved successfully",
                orders_json,
            ))
        }
        Err(e) => {
            Ok(RedisResponse::new(
                500,
                false,
                format!("Failed to get open orders: {}", e),
                serde_json::json!(null),
            ))
        }
    }
}

async fn handle_get_order_status(data: Value, orderbook: &Orderbook) -> Result<RedisResponse<Value>, String> {
    let req: GetOrderStatusRequest = serde_json::from_value(data)
        .map_err(|e| format!("Invalid request data: {}", e))?;

    match orderbook.get_order_status(req.order_id).await {
        Ok(order) => {
            let order_json = serde_json::to_value(&order)
                .map_err(|e| format!("Failed to serialize order: {}", e))?;
            Ok(RedisResponse::new(
                200,
                true,
                "Order status retrieved successfully",
                order_json,
            ))
        }
        Err(e) => {
            Ok(RedisResponse::new(
                404,
                false,
                format!("Order not found: {}", e),
                serde_json::json!(null),
            ))
        }
    }
}

async fn handle_get_order_history(data: Value, _orderbook: &Orderbook) -> Result<RedisResponse<Value>, String> {
    let _req: GetOrderHistoryRequest = serde_json::from_value(data)
        .map_err(|e| format!("Invalid request data: {}", e))?;

    Ok(RedisResponse::new(
        200,
        true,
        "Order history retrieved successfully",
        serde_json::json!([]),
    ))
}

async fn handle_create_user(data: Value, orderbook: &Orderbook) -> Result<RedisResponse<Value>, String> {
    let req: CreateUserRequest = serde_json::from_value(data)
        .map_err(|e| format!("Invalid request data: {}", e))?;

    let user = User {
        id: req.id,
        name: req.name,
        email: req.email,
        balance: req.balance,
        positions: HashMap::new(),
    };

    match orderbook.add_user(user.clone()).await {
        Some(_) => {
            let user_json = serde_json::to_value(&user)
                .map_err(|e| format!("Failed to serialize user: {}", e))?;
            Ok(RedisResponse::new(
                200,
                true,
                "User created successfully",
                user_json,
            ))
        }
        None => {
            Ok(RedisResponse::new(
                500,
                false,
                "Failed to create user",
                serde_json::json!(null),
            ))
        }
    }
}

async fn handle_get_balance(data: Value, orderbook: &Orderbook) -> Result<RedisResponse<Value>, String> {
    let req: GetBalanceRequest = serde_json::from_value(data)
        .map_err(|e| format!("Invalid request data: {}", e))?;

    match orderbook.get_balance(req.user_id).await {
        Ok(balance) => {
            Ok(RedisResponse::new(
                200,
                true,
                "Balance retrieved successfully",
                serde_json::json!({ "balance": balance }),
            ))
        }
        Err(e) => {
            Ok(RedisResponse::new(
                404,
                false,
                format!("Failed to get balance: {}", e),
                serde_json::json!(null),
            ))
        }
    }
}

async fn handle_onramp(data: Value, orderbook: &Orderbook) -> Result<RedisResponse<Value>, String> {
    let req: OnrampRequest = serde_json::from_value(data)
        .map_err(|e| format!("Invalid request data: {}", e))?;

    if req.amount <= 0 {
        return Ok(RedisResponse::new(
            400,
            false,
            "Amount must be greater than 0",
            serde_json::json!(null),
        ));
    }

    match orderbook.update_balance(req.user_id, req.amount).await {
        Ok(_) => {
            match orderbook.get_balance(req.user_id).await {
                Ok(new_balance) => {
                    Ok(RedisResponse::new(
                        200,
                        true,
                        "Balance updated successfully",
                        serde_json::json!({ "balance": new_balance }),
                    ))
                }
                Err(e) => {
                    Ok(RedisResponse::new(
                        500,
                        false,
                        format!("Balance updated but failed to retrieve: {}", e),
                        serde_json::json!(null),
                    ))
                }
            }
        }
        Err(e) => {
            Ok(RedisResponse::new(
                400,
                false,
                format!("Failed to update balance: {}", e),
                serde_json::json!(null),
            ))
        }
    }
}

async fn handle_get_positions(data: Value, orderbook: &Orderbook) -> Result<RedisResponse<Value>, String> {
    let user_id = data["user_id"].as_u64()
        .ok_or_else(|| "Missing or invalid user_id".to_string())?;

    match orderbook.get_user_positions(user_id).await {
        Ok(positions) => {
            let positions_vec: Vec<_> = positions.iter().map(|(market_id, qty)| {
                json!({
                    "market_id": market_id,
                    "quantity": qty
                })
            }).collect();
            Ok(RedisResponse::new(
                200,
                true,
                "Positions retrieved successfully",
                serde_json::json!({ "positions": positions_vec }),
            ))
        }
        Err(e) => {
            Ok(RedisResponse::new(
                500,
                false,
                format!("Failed to get positions: {}", e),
                serde_json::json!(null),
            ))
        }
    }
}

async fn handle_get_position(data: Value, orderbook: &Orderbook) -> Result<RedisResponse<Value>, String> {
    let user_id = data["user_id"].as_u64()
        .ok_or_else(|| "Missing or invalid user_id".to_string())?;
    let market_id = data["market_id"].as_u64()
        .ok_or_else(|| "Missing or invalid market_id".to_string())?;

    match orderbook.get_position(user_id, market_id).await {
        Ok(position) => {
            Ok(RedisResponse::new(
                200,
                true,
                "Position retrieved successfully",
                serde_json::json!({
                    "market_id": market_id,
                    "quantity": position
                }),
            ))
        }
        Err(e) => {
            Ok(RedisResponse::new(
                404,
                false,
                format!("Failed to get position: {}", e),
                serde_json::json!(null),
            ))
        }
    }
}

async fn handle_get_portfolio(data: Value, orderbook: &Orderbook) -> Result<RedisResponse<Value>, String> {
    let user_id = data["user_id"].as_u64()
        .ok_or_else(|| "Missing or invalid user_id".to_string())?;

    let positions = match orderbook.get_user_positions(user_id).await {
        Ok(p) => p,
        Err(e) => {
            return Ok(RedisResponse::new(
                500,
                false,
                format!("Failed to get positions: {}", e),
                serde_json::json!(null),
            ));
        }
    };

    let balance = match orderbook.get_balance(user_id).await {
        Ok(b) => b,
        Err(e) => {
            return Ok(RedisResponse::new(
                500,
                false,
                format!("Failed to get balance: {}", e),
                serde_json::json!(null),
            ));
        }
    };

    let mut total_value = balance as u64;
    let mut positions_with_value = Vec::new();

    for (market_id, quantity) in &positions {
        let best_bid = orderbook.best_bid(*market_id).await.unwrap_or(0);
        let best_ask = orderbook.best_ask(*market_id).await.unwrap_or(0);
        let market_price = if best_bid > 0 && best_ask > 0 {
            (best_bid + best_ask) / 2
        } else if best_bid > 0 {
            best_bid
        } else if best_ask > 0 {
            best_ask
        } else {
            0
        };

        let position_value = *quantity * market_price;
        total_value += position_value;

        positions_with_value.push(json!({
            "market_id": market_id,
            "quantity": quantity,
            "market_price": market_price,
            "value": position_value
        }));
    }

    Ok(RedisResponse::new(
        200,
        true,
        "Portfolio retrieved successfully",
        serde_json::json!({
            "balance": balance,
            "total_value": total_value,
            "positions": positions_with_value,
            "pnl": total_value as i64 - balance
        }),
    ))
}

async fn handle_split_order(data: Value, orderbook: &Orderbook) -> Result<RedisResponse<Value>, String> {
    let req: SplitOrderRequest = serde_json::from_value(data)
        .map_err(|e| format!("Invalid request data: {}", e))?;

    match orderbook.create_split_postion(req.user_id, req.market1_id, req.market2_id, req.amount).await {
        Ok(_) => {
            Ok(RedisResponse::new(
                200,
                true,
                "Split order executed successfully",
                serde_json::json!({ "market1_id": req.market1_id, "market2_id": req.market2_id, "amount": req.amount }),
            ))
        }
        Err(e) => {
            Ok(RedisResponse::new(
                400,
                false,
                format!("Failed to execute split order: {}", e),
                serde_json::json!(null),
            ))
        }
    }
}

async fn handle_merge_order(data: Value, orderbook: &Orderbook) -> Result<RedisResponse<Value>, String> {
    let req: MergeOrderRequest = serde_json::from_value(data)
        .map_err(|e| format!("Invalid request data: {}", e))?;

    match orderbook.merge_position(req.user_id, req.market1_id, req.market2_id).await {
        Ok(_) => {
            Ok(RedisResponse::new(
                200,
                true,
                "Merge order executed successfully",
                serde_json::json!({ "market1_id": req.market1_id, "market2_id": req.market2_id }),
            ))
        }
        Err(e) => {
            Ok(RedisResponse::new(
                400,
                false,
                format!("Failed to execute merge order: {}", e),
                serde_json::json!(null),
            ))
        }
    }
}

async fn handle_init_event_markets(data: Value, orderbook: &Orderbook) -> Result<RedisResponse<Value>, String> {
    let req: InitEventMarketsRequest = serde_json::from_value(data)
        .map_err(|e| format!("Invalid request data: {}", e))?;

    let metas: Vec<MarketMeta> = req.outcomes.into_iter().map(|outcome| {
        MarketMeta {
            event_id: req.event_id,
            outcome_id: outcome.outcome_id,
            yes_market_id: outcome.yes_market_id,
            no_market_id: outcome.no_market_id,
        }
    }).collect();

    match orderbook.init_markets(metas).await {
        Ok(_) => {
            Ok(RedisResponse::new(
                200,
                true,
                "Markets initialized successfully",
                serde_json::json!({ "event_id": req.event_id }),
            ))
        }
        Err(e) => {
            Ok(RedisResponse::new(
                400,
                false,
                format!("Failed to init markets: {}", e),
                serde_json::json!(null),
            ))
        }
    }
}

async fn handle_close_event_markets(data: Value, orderbook: &Orderbook) -> Result<RedisResponse<Value>, String> {
    let req: CloseEventMarketsRequest = serde_json::from_value(data)
        .map_err(|e| format!("Invalid request data: {}", e))?;

    match orderbook.close_event_markets(req.event_id, req.winning_outcome_id).await {
        Ok(_) => {
            Ok(RedisResponse::new(
                200,
                true,
                "Event markets closed successfully",
                serde_json::json!({ "event_id": req.event_id, "winning_outcome_id": req.winning_outcome_id }),
            ))
        }
        Err(e) => {
            Ok(RedisResponse::new(
                400,
                false,
                format!("Failed to close event markets: {}", e),
                serde_json::json!(null),
            ))
        }
    }
}

async fn send_response(request_id: String, response: RedisResponse<Value>) -> Result<(), String> {
    let redis_manager = RedisManager::global()
        .ok_or_else(|| "Redis manager not initialized".to_string())?;

    let response_json = serde_json::to_string(&response)
        .map_err(|e| format!("Failed to serialize response: {}", e))?;

    redis_manager
        .stream_add(
            "engine_responses",
            &[
                ("request_id", &request_id),
                ("data", &response_json),
            ],
        )
        .await
        .map_err(|e| format!("Failed to send response to stream: {}", e))?;

    Ok(())
}

