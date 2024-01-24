use diesel::prelude::*;
use uuid::Uuid;

use super::{customer::Customer, product::Product};

#[derive(Queryable, Selectable, Associations, Identifiable)]
#[diesel(table_name = crate::schema::carts)]
#[diesel(belongs_to(Customer))]
#[diesel(belongs_to(Product))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Cart {
    id: i32,
    uuid: String,
    product_id: i32,
    customer_id: i32,
    quantity: f64,
    created_on: String,
}

impl Cart {
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }
    pub fn get_product_id(&self) -> i32 {
        self.product_id
    }
    pub fn get_customer_id(&self) -> i32 {
        self.customer_id
    }
    pub fn get_quantity(&self) -> f64 {
        self.quantity
    }
    pub fn get_created_on(&self) -> &str {
        &self.created_on
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::carts)]
pub struct NewCartItem {
    uuid: String,
    product_id: i32,
    customer_id: i32,
    quantity: f64,
    created_on: String,
}

impl NewCartItem {
    pub fn new(product: &Product, customer: &Customer, quantity: f64, created_on: String) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            customer_id: customer.get_id(),
            product_id: product.get_id(),
            quantity,
            created_on,
        }
    }
}
