use chrono::Days;
use diesel::prelude::*;
use uuid::Uuid;

use crate::base_types::delivery_status::DeliveryStatus;

use super::customer::Customer;

#[derive(Queryable, Selectable, Associations, Identifiable)]
#[diesel(table_name = crate::schema::orders)]
#[diesel(belongs_to(Customer))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Order {
    id: i32,
    uuid: String,
    created_on: String,
    fulfilled_on: String,
    delivery_charge: f64,
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

    pub fn get_created_on(&self) -> &str {
        &self.created_on
    }

    pub fn get_fulfilled_on(&self) -> &str {
        &self.fulfilled_on
    }

    pub fn get_total_price(&self) -> f64 {
        self.total_price
    }

    pub fn get_delivery_charge(&self) -> f64 {
        self.delivery_charge
    }

    pub fn get_delivery_location(&self) -> &str {
        &self.delivery_location
    }

    pub fn get_delivery_status(&self) -> &str {
        &self.delivery_status
    }

    pub fn get_customer_id(&self) -> i32 {
        self.customer_id
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::orders)]
pub struct NewOrder {
    uuid: String,
    created_on: String,
    fulfilled_on: String,
    delivery_charge: f64,
    delivery_location: String,
    delivery_status: String,
    total_price: f64,
    customer_id: i32,
}

impl NewOrder {
    pub fn new(
        customer: &Customer,
        created_on: String,
        delivery_charge: f64,
        delivery_status: DeliveryStatus,
        delivery_location: String,
        order_total: f64,
    ) -> Self {
        let created_on_clone = created_on.clone();
        Self {
            uuid: Uuid::new_v4().to_string(),
            created_on,
            delivery_charge,
            fulfilled_on: Self::get_delivery_duration(&created_on_clone),
            delivery_location,
            delivery_status: delivery_status.value().to_owned(),
            customer_id: customer.get_id(),
            total_price: order_total + delivery_charge,
        }
    }

    pub fn get_delivery_duration(created_on: &String) -> String {
        let created_on_dt: chrono::NaiveDateTime =
            chrono::NaiveDateTime::parse_from_str(created_on, "%Y-%m-%d %H:%M:%S")
                .expect("Error parsing order date");
        let fulfillment_dt = created_on_dt.checked_add_days(Days::new(1));
        fulfillment_dt.unwrap().to_string()
    }
}
