use actix_web::{get, HttpResponse, Responder};

use crate::base_types::{
    delivery_status::DeliveryStatus, order_status::OrderStatus, payment_method::PaymentMethod,
    product_sku::ProductSKU, shipment_status::ShipmentStatus,
};

#[get("/order-status")]
pub async fn get_order_status() -> impl Responder {
    let statuses: Vec<String> = OrderStatus::all()
        .iter()
        .map(|status| status.value().to_string())
        .collect();

    HttpResponse::Ok().json(statuses)
}
#[get("/delivery-status")]
pub async fn get_delivery_status() -> impl Responder {
    let statuses: Vec<String> = DeliveryStatus::all()
        .iter()
        .map(|status| status.value().to_string())
        .collect();

    HttpResponse::Ok().json(statuses)
}

#[get("/payment-methods")]
pub async fn get_payment_methods() -> impl Responder {
    let methods: Vec<String> = PaymentMethod::all()
        .iter()
        .map(|method| method.value().to_string())
        .collect();

    HttpResponse::Ok().json(methods)
}

#[get("/shipment-status")]
pub async fn get_shipment_status() -> impl Responder {
    let statuses: Vec<String> = ShipmentStatus::all()
        .iter()
        .map(|status| status.value().to_string())
        .collect();

    HttpResponse::Ok().json(statuses)
}

#[get("/product-sku")]
pub async fn get_product_sku() -> impl Responder {
    let units: Vec<String> = ProductSKU::all()
        .iter()
        .map(|unit| unit.value().to_string())
        .collect();

    HttpResponse::Ok().json(units)
}
