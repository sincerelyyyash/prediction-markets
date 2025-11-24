use crate::utils::redis_stream::send_request_and_wait;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use redis_client::RedisRequest;
use serde_json::json;
use std::{
    future::{ready, Ready},
    rc::Rc,
};
use uuid::Uuid;

pub struct AdminMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AdminMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AdminMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AdminMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct AdminMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AdminMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let user_id = match req.extensions().get::<i64>() {
            Some(id) => *id,
            None => {
                return Box::pin(async {
                    Err(ErrorUnauthorized(json!({
                        "status": "error",
                        "message": "Authentication required"
                    })))
                });
            }
        };

        let request_id = Uuid::new_v4().to_string();
        let admin_check_request = RedisRequest::new(
            "db_worker",
            "get_admin_by_id",
            "Check if user is admin",
            json!({
                "id": user_id,
            }),
        );

        let service = self.service.clone();
        Box::pin(async move {
            match send_request_and_wait(request_id, admin_check_request, 5).await {
                Ok(response) => {
                    if response.status_code == 404 || !response.success {
                        return Err(ErrorUnauthorized(json!({
                            "status": "error",
                            "message": "Admin access required"
                        })));
                    }
                    let res = service.call(req).await?;
                    Ok(res)
                }
                Err(_) => Err(ErrorUnauthorized(json!({
                    "status": "error",
                    "message": "Failed to verify admin status"
                }))),
            }
        })
    }
}
