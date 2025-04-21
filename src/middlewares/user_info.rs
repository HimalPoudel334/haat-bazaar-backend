use std::collections::HashSet;

use actix_web::{dev::Payload, error::ErrorUnauthorized, Error, FromRequest, HttpMessage, HttpRequest};
use futures_util::future::{ready, Ready};


#[derive(Clone, Debug)]
pub struct UserInfo {
    pub user_id: String,
    pub roles: HashSet<String>,
}

impl FromRequest for UserInfo {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        if let Some(user_info) = req.extensions().get::<UserInfo>() {
            ready(Ok(user_info.clone()))
        } else {
            ready(Err(ErrorUnauthorized("Not authenticated or user data unavailable")))
        }
    }
}