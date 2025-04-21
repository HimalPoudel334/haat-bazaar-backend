// use actix_web::{Error, HttpMessage};
// use actix_web::error::ErrorUnauthorized;
// use actix_web::{http, web, FromRequest};
// use std::future::{ready, Ready};

// use crate::config::ApplicationConfiguration;
// use crate::utils::jwt_helper::verify_jwt;

// pub struct AuthenticatedUser(pub String);

// impl FromRequest for AuthenticatedUser {
//     type Error = Error;
//     type Future = Ready<Result<Self, Self::Error>>;

//     fn from_request(
//         req: &actix_web::HttpRequest,
//         _payload: &mut actix_web::dev::Payload,
//     ) -> Self::Future {
//         let app_config = req
//             .app_data::<web::Data<ApplicationConfiguration>>()
//             .unwrap();

//         //get the request and extract the token
//         let token = 
//                 req.headers()
//                     .get(http::header::AUTHORIZATION)
//                     .map(|h| h.to_str().unwrap().split_at(7).1.to_string()); //remove the "Bearer " part of the token

//         if token.is_none() {
//             return ready(Err(ErrorUnauthorized(serde_json::json!({"error":"Authorization has been denied for this request"}))));
//         }

//         let claims = match verify_jwt(&token.unwrap(), app_config.jwt_secret.to_owned()) {
//             Ok(decoded) => decoded,
//             Err(_e) => {
//                 return ready(Err(ErrorUnauthorized(serde_json::json!({"error":"Invalid token"}))));
//             }
//         };

//         let user_id = claims.sub.as_str().to_owned();
//         //insert the user id to request header
//         req.extensions_mut().insert(user_id.clone());

//         ready(Ok(AuthenticatedUser(user_id)))
//     }
// }

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::AUTHORIZATION,
    web, Error as ActixWebError, HttpMessage,
};
use futures_util::future::{ok, FutureExt, LocalBoxFuture, Ready};
use log::{debug, error, warn};
use std::rc::Rc; 

use crate::{config::ApplicationConfiguration, middlewares::user_info::UserInfo};
use crate::utils::jwt_helper;

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
        debug!("Running JWT Authentication Middleware (Transform)...");

        let service = Rc::clone(&self.service);

        let config_opt = req.app_data::<web::Data<ApplicationConfiguration>>().cloned(); // Clone Data<Arc<T>>

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
                debug!("Bearer token found.");
                match jwt_helper::verify_jwt(token_str, &config.jwt_secret.as_bytes()) {
                    Ok(claims) => {
                        debug!("JWT verified successfully. Claims: {:?}", claims);
                        let user_info = UserInfo {
                            user_id: claims.sub.clone(),
                            roles: claims.roles.clone(),
                        };
                        req.extensions_mut().insert(user_info);
                        debug!("UserInfo inserted into request extensions.");
                    }
                    Err(e) => {
                        warn!("JWT verification failed: {}", e);
                    }
                }
            } else {
                debug!("No valid Bearer token found in Authorization header.");
            }

            service.call(req).await
        }
        .boxed_local()
    }
}