use uuid::Uuid;

use super::{order::Order, product::Product};

pub struct OrderDetails {
    id: u32,
    uuid: String,
    quantity: f64,
    product_id: i32,
    order_id: i32,
    price: f64,
}

pub struct NewOrderDetail {
    uuid: String,
    quantity: f64,
    product_id: u32,
    order_id: u32,
}

impl NewOrderDetail {
    pub fn new(quantity: f64, product: Product, order: Order) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            quantity,
            product_id: product.get_id(),
            order_id: order.get_id(),
        }
    }
}
