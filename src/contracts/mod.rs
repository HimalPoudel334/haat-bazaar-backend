use actix_web::HttpResponse;

pub mod admin_device;
pub mod auth;
pub mod cart;
pub mod category;
pub mod invoice;
pub mod invoice_item;
pub mod khalti_payment;
pub mod order;
pub mod order_item;
pub mod payment;
pub mod product;
pub mod product_image;
pub mod shipment;
pub mod user;

pub struct ResponseWrapper {
    pub success: bool,
    pub response: HttpResponse,
}
