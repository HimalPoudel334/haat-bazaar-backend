use actix_web::web;

use crate::handlers::{auth, cart, category, invoice, order, order_details, payment, product, user};

pub fn app_routes(cfg: &mut web::ServiceConfig) {
    cfg
    .service(
        web::scope("/auth")
            .service(auth::login),
    )
    .service(
        web::scope("/users")
            .service(user::create)
            .service(user::get)
            .service(user::get_user)
            .service(user::get_user_from_phone_number)
            .service(user::get_user_from_email)
            .service(user::edit)
            .service(user::delete),
    )
    .service(
        web::scope("/products")
            .service(product::get)
            .service(product::create)
            .service(product::get_product)
            .service(product::edit)
            .service(product::upload_product_images)
            .service(product::get_product_images_list)
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
            .service(order::get_order)
            .service(order::get_orders)
            .service(order::get_user_orders)
            .service(order::update_delivery_status),
    )
    .service(
        web::scope("/order-details")
            .service(order_details::get)
            .service(order_details::get_order_detail)
            .service(order_details::add_order_detail),
    )
    .service(
        web::scope("/carts")
            .service(cart::get)
            .service(cart::create)
            .service(cart::update_quantity),
    )
    .service(
        web::scope("/payments")
            .service(payment::get)
            .service(payment::create)
            .service(payment::esewa_payment_confirmation)
            .service(payment::khalti_payment_get_pidx)
            .service(payment::khalti_payment_confirmation),
    )
    .service(
        web::scope("/invoices")
            .service(invoice::create)
            .service(invoice::get)
            .service(invoice::get_all),
    );
}
