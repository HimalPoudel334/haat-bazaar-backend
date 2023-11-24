use diesel::prelude::*;

use super::order::Order;

#[derive(Queryable, Selectable, Associations)]
#[diesel(table_name = crate::schema::shipments)]
#[diesel(belongs_to(Order))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Shipment {
    id: i32,
    uuid: String,
    ship_date: String,
    address: String,
    city: String,
    state: String,
    country: String,
    order_id: i32,
}
