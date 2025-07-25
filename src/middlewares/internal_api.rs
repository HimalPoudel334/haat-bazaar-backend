use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use std::rc::Rc;
use std::task::{Context, Poll};

pub struct InternalOnly;

impl<S, B> Transform<S, ServiceRequest> for InternalOnly
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static, // Add 'static bound for B, as ServiceResponse<B> needs to be 'static
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = InternalOnlyMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(InternalOnlyMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct InternalOnlyMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for InternalOnlyMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static, // Add 'static bound for B
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let peer_addr = req
            .connection_info()
            .realip_remote_addr()
            .map(str::to_string)
            .unwrap_or_default();

        if peer_addr.starts_with("127.0.0.1") || peer_addr.starts_with("[::1]") {
            let fut = self.service.call(req);
            Box::pin(async move { fut.await })
        } else {
            Box::pin(async move {
                Err(actix_web::error::ErrorForbidden(
                    "Access denied: internal API only",
                ))
            })
        }
    }
}

