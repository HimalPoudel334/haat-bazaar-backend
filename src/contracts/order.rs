use diesel::{deserialize::Queryable, Selectable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Selectable, Queryable)]
#[diesel(table_name = crate::schema::orders)]
pub struct Order {
    pub uuid: String,
    pub created_on: String,
    pub fulfilled_on: String,
    pub delivery_location: String,
    pub delivery_status: String,
    pub total_price: f64,
    pub customer_id: String,
}

#[derive(Deserialize)]
pub struct OrderCreate {
    pub created_on: String,
    pub fulfilled_on: String,
    pub delivery_location: String,
    pub delivery_status: String,
    pub total_price: f64,
    pub customer_id: String,
}

#[derive(Deserialize)]
pub struct OrderEdit {
    pub fulfilled_on: String,
    pub delivery_status: String,
    pub delivery_location: String,
    pub total_price: f64,
    pub customer_id: String,
}

#[derive(Deserialize)]
pub struct OrderDeliveryStatus {
    pub delivery_status: String,
}
