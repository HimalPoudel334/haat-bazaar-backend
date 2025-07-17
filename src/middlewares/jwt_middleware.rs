use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::AUTHORIZATION,
    web, Error as ActixWebError, HttpMessage,
};
use futures_util::future::{ok, FutureExt, LocalBoxFuture, Ready};
use log::{debug, error, warn};
use std::rc::Rc;

use crate::utils::jwt_helper;
use crate::{config::ApplicationConfiguration, middlewares::user_info::UserInfo};

#[derive(Clone, Copy)]
pub struct JwtAuthentication;

impl<S, B> Transform<S, ServiceRequest> for JwtAuthentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixWebError> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = ActixWebError;
    type InitError = ();
    type Transform = JwtAuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtAuthenticationMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct JwtAuthenticationMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixWebError> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = ActixWebError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        let config_opt = req
            .app_data::<web::Data<ApplicationConfiguration>>()
            .cloned(); // Clone Data<Arc<T>>

        async move {
            let config = match config_opt {
                Some(cfg) => cfg,
                None => {
                    error!("ApplicationConfiguration not found in app_data");
                    return Err(actix_web::error::ErrorInternalServerError(
                        "Server configuration error preventing authentication",
                    ));
                }
            };

            // --- Extract token ---
            let token = req
                .headers()
                .get(AUTHORIZATION)
                .and_then(|hv| hv.to_str().ok())
                .filter(|s| s.starts_with("Bearer "))
                .map(|s| &s[7..]);

            if let Some(token_str) = token {
                match jwt_helper::verify_jwt(token_str, &config.jwt_secret.as_bytes()).await {
                    Ok(claims) => {
                        let user_info = UserInfo {
                            user_id: claims.sub.clone(),
                            roles: claims.roles.clone(),
                        };
                        req.extensions_mut().insert(user_info);
                    }
                    Err(e) => {
                        println!("JWT verification failed: {}", e);
                    }
                }
            } else {
                println!("No valid Bearer token found in Authorization header.");
            }

            service.call(req).await
        }
        .boxed_local()
    }
}
