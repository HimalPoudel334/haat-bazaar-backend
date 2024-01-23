use diesel::prelude::*;

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
