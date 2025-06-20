use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::{ErrorForbidden, ErrorUnauthorized},
    Error as ActixWebError, HttpMessage,
};
use futures_util::future::{ok, FutureExt, LocalBoxFuture, Ready};
use std::collections::HashSet;
use std::rc::Rc;

use super::user_info::UserInfo;

#[derive(Clone)]
struct AuthConfig {
    required_roles: Option<HashSet<String>>,
}

// --- Authorization Middleware Factory ---
#[derive(Clone)]
pub struct Auth {
    config: AuthConfig,
}

impl Auth {
    pub fn authenticated() -> Self {
        Auth {
            config: AuthConfig {
                required_roles: None,
            },
        }
    }

    pub fn require_roles(roles: &[&str]) -> Self {
        Auth {
            config: AuthConfig {
                required_roles: Some(roles.iter().map(|s| s.to_string()).collect()),
            },
        }
    }
}

// --- Transform Implementation (Factory) ---
impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixWebError> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = ActixWebError;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddleware {
            service: Rc::new(service),
            config: self.config.clone(),
        })
    }
}

// --- The Actual Authorization Middleware Service ---
pub struct AuthMiddleware<S> {
    service: Rc<S>,
    config: AuthConfig,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
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
        let user_opt = req.extensions().get::<UserInfo>().cloned();

        let service = Rc::clone(&self.service);
        let config = self.config.clone();

        async move {
            match user_opt {
                Some(user_info) => {
                    // Check if specific roles are required
                    if let Some(required_roles) = &config.required_roles {
                        // Do this if one user has multiple roles
                        // if required_roles.is_subset(&user_info.roles) {
                        if user_info.roles.is_subset(&required_roles) {
                            service.call(req).await
                        } else {
                            Err(ErrorForbidden(
                                serde_json::json!({"message":"Insufficient permissions"}),
                            ))
                        }
                    } else {
                        service.call(req).await
                    }
                }
                None => Err(ErrorUnauthorized(
                    serde_json::json!({"message":"Authentication required"}),
                )),
            }
        }
        .boxed_local()
    }
}
