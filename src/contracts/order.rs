use diesel::{deserialize::Queryable, Selectable};
use serde::{Deserialize, Serialize};

use super::order_details::NewOrderDetail;

#[derive(Serialize, Deserialize, Selectable, Queryable)]
#[diesel(table_name = crate::schema::orders)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    #[serde(rename = "id")]
    pub uuid: String,
    pub created_on: String,
    pub fulfilled_on: String,
    pub delivery_location: String,
    pub delivery_status: String,
    pub total_price: f64,
    pub customer_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderCreate {
    pub created_on: String,
    pub fulfilled_on: Option<String>,
    pub delivery_location: String,
    pub delivery_status: String,
    pub total_price: f64,
    pub customer_id: String,
    pub order_details: Vec<NewOrderDetail>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderEdit {
    pub fulfilled_on: String,
    pub delivery_status: String,
    pub delivery_location: String,
    pub total_price: f64,
    pub customer_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderDeliveryStatus {
    pub delivery_status: String,
}
