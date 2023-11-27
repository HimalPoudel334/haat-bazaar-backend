use actix_web::web;

use crate::handlers::{category, customer, product};

pub fn app_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/customers")
            .service(customer::create)
            .service(customer::get_customer),
    )
    .service(web::scope("/products").service(product::create))
    .service(
        web::scope("/categories")
            .service(category::create)
            .service(category::get)
            .service(category::get_category)
            .service(category::edit)
            .service(category::delete),
    );
}
