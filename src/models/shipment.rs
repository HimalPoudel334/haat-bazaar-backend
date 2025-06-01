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
    zip_code: String,
    order_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::shipments)]
pub struct NewShipment {
    uuid: String,
    ship_date: String,
    address: String,
    city: String,
    state: String,
    country: String,
    zip_code: String,
    order_id: i32,
}

impl NewShipment {
    pub fn new(address: &str, order: &Order) -> Self {
        let parts: Vec<&str> = address.split(',').map(|s| s.trim()).collect();
        //address should in the below format address,city,state,country

        let address_part = parts.get(0).unwrap_or(&"").to_string();
        let zip_code = parts.get(1).unwrap_or(&"").to_string();
        let city_part = parts.get(2).unwrap_or(&"").to_string();
        let state_part = parts.get(3).unwrap_or(&"").to_string();
        let country_part = parts.get(4).unwrap_or(&"").to_string();

        Self {
            uuid: uuid::Uuid::new_v4().to_string(),
            ship_date: order.get_fulfilled_on().to_string(),
            address: address_part,
            city: city_part,
            state: state_part,
            country: country_part,
            zip_code: zip_code,
            order_id: order.get_id(),
        }
    }
}
