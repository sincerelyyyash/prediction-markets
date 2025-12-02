use actix_web::HttpResponse;
use redis_client::RedisResponse;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;

pub fn build_json_from_redis_response<T>(response: &RedisResponse<T>) -> HttpResponse
where
    T: Serialize + DeserializeOwned + Clone,
{
    if response.status_code >= 400 {
        let status = actix_web::http::StatusCode::from_u16(response.status_code as u16)
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);

        return HttpResponse::build(status).json(json!({
            "status": if response.success { "success" } else { "error" },
            "message": response.message,
            "data": response.data
        }));
    }

    HttpResponse::Ok().json(response.data.clone())
}
