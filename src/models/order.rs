use diesel::prelude::*;

use super::customer::Customer;

#[derive(Queryable, Selectable, Associations)]
#[diesel(table_name = crate::schema::orders)]
#[diesel(belongs_to(Customer))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Order {
    id: i32,
    uuid: String,
    created_on: String,
    fulfilled_on: String,
    delivery_location: String,
    delivery_status: String,
    total_price: f64,
    customer_id: i32,
}

impl Order {
    pub fn get_id(&self) -> i32 {
        self.id
    }
    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }
}
