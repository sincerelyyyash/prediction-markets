use crate::types::order_types::{
    MergeOrderInput, ModifyOrderInput, OrderSideInput, OrderTypeInput, PlaceOrderInput,
    SplitOrderInput,
};
use crate::utils::jwt::extract_user_id;
use crate::utils::redis_stream::send_request_and_wait;
use actix_web::{get, post, put, web, HttpRequest, HttpResponse, Responder};
use redis_client::RedisRequest;
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

#[post("/orders")]
pub async fn place_order(req: HttpRequest, body: web::Json<PlaceOrderInput>) -> impl Responder {
    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": e.to_string()
        }));
    }

    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let order_side = match body.side {
        OrderSideInput::Ask => "Ask",
        OrderSideInput::Bid => "Bid",
    };

    let order_type_str = match body.order_type {
        OrderTypeInput::Market => "Market",
        OrderTypeInput::Limit => "Limit",
    };

    if body.order_type == OrderTypeInput::Limit && body.price.is_none() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Price is required for limit orders"
        }));
    }

    let order_data = json!({
        "market_id": body.market_id,
        "user_id": user_id as u64,
        "price": body.price,
        "original_qty": body.quantity,
        "remaining_qty": body.quantity,
        "side": order_side,
        "order_type": order_type_str,
    });

    let request_id = Uuid::new_v4().to_string();
    let redis_request =
        RedisRequest::new("engine", "place-order", "Place order request", order_data);

    match send_request_and_wait(request_id, redis_request, 10).await {
        Ok(response) => {
            if response.status_code >= 400 {
                let status = actix_web::http::StatusCode::from_u16(response.status_code as u16)
                    .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
                return HttpResponse::build(status).json(json!({
                    "status": if response.success { "success" } else { "error" },
                    "message": response.message,
                    "data": response.data
                }));
            }
            HttpResponse::Ok().json(json!({
                "status": "success",
                "message": response.message,
                "data": response.data
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to process order",
            "error": e
        })),
    }
}

#[post("/orders/{order_id}/cancel")]
pub async fn cancel_order(req: HttpRequest, path: web::Path<u64>) -> impl Responder {
    let order_id = path.into_inner();

    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let order_data = json!({
        "order_id": order_id,
        "user_id": user_id as u64,
    });

    let request_id = Uuid::new_v4().to_string();
    let redis_request =
        RedisRequest::new("engine", "cancel-order", "Cancel order request", order_data);

    match send_request_and_wait(request_id, redis_request, 10).await {
        Ok(response) => {
            if response.status_code >= 400 {
                let status = actix_web::http::StatusCode::from_u16(response.status_code as u16)
                    .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
                return HttpResponse::build(status).json(json!({
                    "status": if response.success { "success" } else { "error" },
                    "message": response.message,
                    "data": response.data
                }));
            }
            HttpResponse::Ok().json(json!({
                "status": "success",
                "message": response.message,
                "data": response.data
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to cancel order",
            "error": e
        })),
    }
}

#[put("/orders/{order_id}")]
pub async fn modify_order(
    req: HttpRequest,
    path: web::Path<u64>,
    body: web::Json<ModifyOrderInput>,
) -> impl Responder {
    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": e.to_string()
        }));
    }

    let order_id = path.into_inner();

    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let mut order_data = json!({
        "order_id": order_id,
        "user_id": user_id as u64,
    });

    if let Some(price) = body.price {
        order_data["price"] = json!(price);
    }
    if let Some(quantity) = body.quantity {
        order_data["original_qty"] = json!(quantity);
        order_data["remaining_qty"] = json!(quantity);
    }

    let request_id = Uuid::new_v4().to_string();
    let redis_request =
        RedisRequest::new("engine", "modify-order", "Modify order request", order_data);

    match send_request_and_wait(request_id, redis_request, 10).await {
        Ok(response) => {
            if response.status_code >= 400 {
                let status = actix_web::http::StatusCode::from_u16(response.status_code as u16)
                    .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
                return HttpResponse::build(status).json(json!({
                    "status": if response.success { "success" } else { "error" },
                    "message": response.message,
                    "data": response.data
                }));
            }
            HttpResponse::Ok().json(json!({
                "status": "success",
                "message": response.message,
                "data": response.data
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to modify order",
            "error": e
        })),
    }
}

#[post("/orders/split")]
pub async fn split_order(req: HttpRequest, body: web::Json<SplitOrderInput>) -> impl Responder {
    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": e.to_string()
        }));
    }

    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let order_data = json!({
        "user_id": user_id as u64,
        "market1_id": body.market1_id,
        "market2_id": body.market2_id,
        "amount": body.amount,
    });

    let request_id = Uuid::new_v4().to_string();
    let redis_request =
        RedisRequest::new("engine", "split-order", "Split order request", order_data);

    match send_request_and_wait(request_id, redis_request, 10).await {
        Ok(response) => {
            if response.status_code >= 400 {
                let status = actix_web::http::StatusCode::from_u16(response.status_code as u16)
                    .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
                return HttpResponse::build(status).json(json!({
                    "status": if response.success { "success" } else { "error" },
                    "message": response.message,
                    "data": response.data
                }));
            }
            HttpResponse::Ok().json(json!({
                "status": "success",
                "message": response.message,
                "data": response.data
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to process split order",
            "error": e
        })),
    }
}

#[post("/orders/merge")]
pub async fn merge_order(req: HttpRequest, body: web::Json<MergeOrderInput>) -> impl Responder {
    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": e.to_string()
        }));
    }

    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let order_data = json!({
        "user_id": user_id as u64,
        "market1_id": body.market1_id,
        "market2_id": body.market2_id,
    });

    let request_id = Uuid::new_v4().to_string();
    let redis_request =
        RedisRequest::new("engine", "merge-order", "Merge order request", order_data);

    match send_request_and_wait(request_id, redis_request, 10).await {
        Ok(response) => {
            if response.status_code >= 400 {
                let status = actix_web::http::StatusCode::from_u16(response.status_code as u16)
                    .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
                return HttpResponse::build(status).json(json!({
                    "status": if response.success { "success" } else { "error" },
                    "message": response.message,
                    "data": response.data
                }));
            }
            HttpResponse::Ok().json(json!({
                "status": "success",
                "message": response.message,
                "data": response.data
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to process merge order",
            "error": e
        })),
    }
}

#[get("/orders")]
pub async fn get_open_orders(req: HttpRequest) -> impl Responder {
    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let order_data = json!({
        "user_id": user_id as u64,
    });

    let request_id = Uuid::new_v4().to_string();
    let redis_request = RedisRequest::new(
        "engine",
        "get-open-orders",
        "Get user open orders",
        order_data,
    );

    match send_request_and_wait(request_id, redis_request, 10).await {
        Ok(response) => {
            if response.status_code >= 400 {
                let status = actix_web::http::StatusCode::from_u16(response.status_code as u16)
                    .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
                return HttpResponse::build(status).json(json!({
                    "status": if response.success { "success" } else { "error" },
                    "message": response.message,
                    "data": response.data
                }));
            }
            HttpResponse::Ok().json(json!({
                "status": "success",
                "message": response.message,
                "data": response.data
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to fetch open orders",
            "error": e
        })),
    }
}

#[get("/orders/{order_id}")]
pub async fn get_order_status(req: HttpRequest, path: web::Path<u64>) -> impl Responder {
    let order_id = path.into_inner();

    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let order_data = json!({
        "order_id": order_id,
        "user_id": user_id as u64,
    });

    let request_id = Uuid::new_v4().to_string();
    let redis_request = RedisRequest::new(
        "db_worker",
        "get_order_by_id",
        "Get order by ID",
        order_data,
    );

    match send_request_and_wait(request_id, redis_request, 10).await {
        Ok(response) => {
            if response.status_code >= 400 {
                if response.status_code == 404 {
                    return HttpResponse::NotFound().json(json!({
                        "status": "error",
                        "message": response.message,
                        "data": response.data
                    }));
                }
                if response.status_code == 403 {
                    return HttpResponse::Forbidden().json(json!({
                        "status": "error",
                        "message": response.message,
                        "data": response.data
                    }));
                }
                let status = actix_web::http::StatusCode::from_u16(response.status_code as u16)
                    .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
                return HttpResponse::build(status).json(json!({
                    "status": if response.success { "success" } else { "error" },
                    "message": response.message,
                    "data": response.data
                }));
            }
            HttpResponse::Ok().json(response.data)
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to fetch order",
            "error": e
        })),
    }
}

#[get("/orders/history")]
pub async fn get_order_history(req: HttpRequest) -> impl Responder {
    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let order_data = json!({
        "user_id": user_id as u64,
    });

    let request_id = Uuid::new_v4().to_string();
    let redis_request = RedisRequest::new(
        "db_worker",
        "get_orders_by_user",
        "Get order history",
        order_data,
    );

    match send_request_and_wait(request_id, redis_request, 10).await {
        Ok(response) => {
            if response.status_code >= 400 {
                let status = actix_web::http::StatusCode::from_u16(response.status_code as u16)
                    .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
                return HttpResponse::build(status).json(json!({
                    "status": if response.success { "success" } else { "error" },
                    "message": response.message,
                    "data": response.data
                }));
            }
            HttpResponse::Ok().json(response.data)
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to fetch order history",
            "error": e
        })),
    }
}

#[get("/orders/user/{user_id}")]
pub async fn get_orders_by_user(req: HttpRequest, path: web::Path<u64>) -> impl Responder {
    let user_id = path.into_inner();

    let request_id = Uuid::new_v4().to_string();
    let read_request = RedisRequest::new(
        "db_worker",
        "get_orders_by_user",
        "Get orders for user",
        json!({
            "user_id": user_id,
        }),
    );

    match send_request_and_wait(request_id, read_request, 10).await {
        Ok(response) => {
            if response.status_code >= 400 {
                let status = actix_web::http::StatusCode::from_u16(response.status_code as u16)
                    .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
                return HttpResponse::build(status).json(json!({
                    "status": if response.success { "success" } else { "error" },
                    "message": response.message,
                    "data": response.data
                }));
            }
            HttpResponse::Ok().json(response.data)
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to fetch orders",
            "error": e
        })),
    }
}

#[get("/orders/market/{market_id}")]
pub async fn get_orders_by_market(req: HttpRequest, path: web::Path<u64>) -> impl Responder {
    let market_id = path.into_inner();

    let request_id = Uuid::new_v4().to_string();
    let read_request = RedisRequest::new(
        "db_worker",
        "get_orders_by_market",
        "Get orders for market",
        json!({
            "market_id": market_id,
        }),
    );

    match send_request_and_wait(request_id, read_request, 10).await {
        Ok(response) => {
            if response.status_code >= 400 {
                let status = actix_web::http::StatusCode::from_u16(response.status_code as u16)
                    .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
                return HttpResponse::build(status).json(json!({
                    "status": if response.success { "success" } else { "error" },
                    "message": response.message,
                    "data": response.data
                }));
            }
            HttpResponse::Ok().json(response.data)
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to fetch orders",
            "error": e
        })),
    }
}
