use diesel::prelude::*;

use super::order::Order;

#[derive(Queryable, Selectable, Associations, Identifiable)]
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
    status: String,
    assigned_to: Option<i32>,
}

impl Shipment {
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }

    pub fn get_ship_date(&self) -> &str {
        &self.ship_date
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }

    pub fn get_city(&self) -> &str {
        &self.city
    }

    pub fn get_state(&self) -> &str {
        &self.state
    }

    pub fn get_country(&self) -> &str {
        &self.country
    }

    pub fn get_zip_code(&self) -> &str {
        &self.zip_code
    }

    pub fn get_order_id(&self) -> i32 {
        self.order_id
    }

    pub fn get_status(&self) -> &str {
        &self.status
    }

    pub fn get_assigned_to(&self) -> Option<i32> {
        self.assigned_to
    }
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
    status: String,
    assigned_to: Option<i32>,
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
            status: order.get_delivery_status().to_string(),
            assigned_to: None,
        }
    }
}
