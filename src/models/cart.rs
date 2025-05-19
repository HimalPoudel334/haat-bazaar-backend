use diesel::prelude::*;
use uuid::Uuid;

use super::{product::Product, user::User};

#[derive(Queryable, Selectable, Associations, Identifiable)]
#[diesel(table_name = crate::schema::carts)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Product))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Cart {
    id: i32,
    uuid: String,
    product_id: i32,
    user_id: i32,
    quantity: f64,
    sku: String,
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
    pub fn get_user_id(&self) -> i32 {
        self.user_id
    }
    pub fn get_quantity(&self) -> f64 {
        self.quantity
    }
    pub fn get_sku(&self) -> &str {
        &self.sku
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
    user_id: i32,
    quantity: f64,
    sku: String,
    created_on: String,
}

impl NewCartItem {
    pub fn new(product: &Product, user: &User, quantity: f64, created_on: String) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            user_id: user.get_id(),
            product_id: product.get_id(),
            quantity,
            sku: product.get_unit().to_owned(),
            created_on,
        }
    }
}
