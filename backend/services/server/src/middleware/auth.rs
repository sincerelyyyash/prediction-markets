use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::{ErrorUnauthorized, ErrorInternalServerError},
    Error, HttpMessage, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use serde_json::json;
use std::{
    future::{ready, Ready},
    rc::Rc,
};
use crate::utils::jwt::verify_jwt;
use std::env;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
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
        let auth_header = req.headers().get("Authorization");

        let user_id = match auth_header {
            Some(header) => {
                let token_str = match header.to_str() {
                    Ok(s) => s,
                    Err(_) => {
                        return Box::pin(async {
                            Err(ErrorUnauthorized(json!({
                                "status": "error",
                                "message": "Invalid Authorization header"
                            })))
                        });
                    }
                };

                let token = match token_str.strip_prefix("Bearer ") {
                    Some(t) => t,
                    None => {
                        return Box::pin(async {
                            Err(ErrorUnauthorized(json!({
                                "status": "error",
                                "message": "Invalid token format"
                            })))
                        });
                    }
                };

                let jwt_secret = match env::var("JWT_SECRET") {
                    Ok(secret) => secret,
                    Err(_) => {
                        return Box::pin(async {
                            Err(ErrorInternalServerError(json!({
                                "status": "error",
                                "message": "JWT secret not configured"
                            })))
                        });
                    }
                };

                match verify_jwt(token, &jwt_secret) {
                    Ok(id) => id,
                    Err(_) => {
                        return Box::pin(async {
                            Err(ErrorUnauthorized(json!({
                                "status": "error",
                                "message": "Invalid or expired token"
                            })))
                        });
                    }
                }
            }
            None => {
                return Box::pin(async {
                    Err(ErrorUnauthorized(json!({
                        "status": "error",
                        "message": "Missing Authorization header"
                    })))
                });
            }
        };

        req.extensions_mut().insert(user_id);

        let service = self.service.clone();
        Box::pin(async move {
            let res = service.call(req).await?;
            Ok(res)
        })
    }
}

