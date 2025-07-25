use chrono::Days;
use diesel::prelude::*;
use uuid::Uuid;

use crate::base_types::{delivery_status::DeliveryStatus, order_status::OrderStatus};

use super::user::User;

#[derive(Queryable, Selectable, Associations, Identifiable)]
#[diesel(table_name = crate::schema::orders)]
#[diesel(belongs_to(User))]
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
    user_id: i32,
    quantity: f64,
    status: String,
    discount: f64,
    amount: f64,
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

    pub fn get_user_id(&self) -> i32 {
        self.user_id
    }

    pub fn get_quantity(&self) -> f64 {
        self.quantity
    }

    pub fn get_status(&self) -> &str {
        &self.status
    }

    pub fn get_discount(&self) -> f64 {
        self.discount
    }

    pub fn get_amount(&self) -> f64 {
        self.amount
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
    quantity: f64,
    user_id: i32,
    status: String,
    discount: f64,
    amount: f64,
}

impl NewOrder {
    pub fn new(
        user: &User,
        created_on: &String,
        delivery_charge: f64,
        delivery_status: DeliveryStatus,
        delivery_location: &String,
        order_total: f64,
        quantity: f64,
        status: OrderStatus,
        discount: f64,
    ) -> Self {
        let created_on_clone = created_on.clone();
        Self {
            uuid: Uuid::new_v4().to_string(),
            created_on: created_on.to_string(),
            delivery_charge,
            fulfilled_on: Self::get_delivery_duration(&created_on_clone),
            delivery_location: delivery_location.to_string(),
            delivery_status: delivery_status.value().to_owned(),
            user_id: user.get_id(),
            total_price: order_total + delivery_charge,
            quantity,
            status: status.value().to_owned(),
            discount,
            amount: order_total + delivery_charge - discount,
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
