use diesel::prelude::*;
use uuid::Uuid;

use super::{order::Order, product::Product};

#[derive(Queryable, Selectable, Associations, Identifiable)]
#[diesel(table_name = crate::schema::order_details)]
#[diesel(belongs_to(Order))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct OrderDetail {
    id: i32,
    uuid: String,
    quantity: f64,
    product_id: i32,
    order_id: i32,
    price: f64,
}

impl OrderDetail {
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }

    pub fn get_quantity(&self) -> f64 {
        self.quantity
    }

    pub fn get_price(&self) -> f64 {
        self.price
    }

    pub fn get_order_id(&self) -> i32 {
        self.order_id
    }

    pub fn get_product_id(&self) -> i32 {
        self.product_id
    }
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
    pub fn new(quantity: f64, product: &Product, order: &Order) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            quantity,
            product_id: product.get_id(),
            order_id: order.get_id(),
            price: quantity * product.get_price(),
        }
    }
}
