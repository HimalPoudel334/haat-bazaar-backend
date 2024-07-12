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
    pub delivery_charge: f64,
    pub delivery_location: String,
    pub delivery_status: String,
    pub total_price: f64,
    pub customer_id: String,
}

/* Order response */
#[derive(Serialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    #[serde(rename = "id")]
    pub uuid: String,
    pub created_on: String,
    pub fulfilled_on: String,
    pub delivery_charge: f64,
    pub delivery_location: String,
    pub delivery_status: String,
    pub total_price: f64,
    pub customer: CustomerResponse,
    pub order_items: Vec<OrderItemResponse>,
}

#[derive(Serialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct CustomerResponse {
    #[serde(rename = "id")]
    pub uuid: String,
    pub first_name: String,
    pub last_name: String,
    pub phone_number: String,
}

#[derive(Serialize, Queryable)]
pub struct OrderItemResponse {
    #[serde(rename = "id")]
    pub uuid: String,
    pub product: ProductResponse,
    pub quantity: f64,
    pub price: f64,
}

#[derive(Serialize, Queryable)]
pub struct ProductResponse {
    #[serde(rename = "id")]
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub image: String,
    pub price: f64,
    pub unit: String,
    pub category: CategoryResponse,
}

#[derive(Serialize, Queryable)]
pub struct CategoryResponse {
    pub uuid: String,
    pub name: String,
}
/* End */

#[derive(Serialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct CustomerOrderResponse {
    #[serde(rename = "id")]
    pub uuid: String,
    pub created_on: String,
    pub fulfilled_on: String,
    pub delivery_charge: f64,
    pub delivery_location: String,
    pub delivery_status: String,
    pub total_price: f64,
    pub order_items: Vec<OrderItemResponse>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderCreate {
    pub created_on: String,
    pub fulfilled_on: Option<String>,
    pub delivery_charge: f64,
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
