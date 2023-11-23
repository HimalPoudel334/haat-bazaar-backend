use actix_web::web;

use crate::handlers::{category, customer, product};

pub fn app_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/customer")
            .service(customer::create)
            .service(customer::get_customer),
    )
    .service(web::scope("/product").service(product::create))
    .service(web::scope("/category").service(category::create));
}
