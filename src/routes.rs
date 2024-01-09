use actix_web::web;

use crate::handlers::{category, customer, order, product};

pub fn app_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/customers")
            .service(customer::create)
            .service(customer::get)
            .service(customer::get_customer)
            .service(customer::get_customer_from_phone_number)
            .service(customer::edit)
            .service(customer::delete),
    )
    .service(
        web::scope("/products")
            .service(product::get)
            .service(product::create)
            .service(product::get_product)
            .service(product::edit)
            .service(product::upload_product_images)
            .service(product::delete),
    )
    .service(
        web::scope("/categories")
            .service(category::create)
            .service(category::get)
            .service(category::get_category)
            .service(category::edit)
            .service(category::delete),
    )
    .service(
        web::scope("/orders")
            .service(order::create)
            .service(order::edit)
            .service(order::get_orders)
            .service(order::update_delivery_status),
    );
}
