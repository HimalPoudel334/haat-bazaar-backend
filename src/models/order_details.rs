use diesel::prelude::*;
use uuid::Uuid;

use super::{order::Order, product::Product};

#[derive(Queryable, Selectable, Associations)]
#[diesel(table_name = crate::schema::order_details)]
#[diesel(belongs_to(Order))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct OrderDetails {
    id: i32,
    uuid: String,
    quantity: f64,
    product_id: i32,
    order_id: i32,
    price: f64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::order_details)]
pub struct NewOrderDetail {
    uuid: String,
    quantity: f64,
    product_id: i32,
    order_id: i32,
    price: f64,
}

impl NewOrderDetail {
    pub fn new(quantity: f64, price: f64, product: &Product, order: &Order) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            quantity,
            product_id: product.get_id(),
            order_id: order.get_id(),
            price,
        }
    }
}
