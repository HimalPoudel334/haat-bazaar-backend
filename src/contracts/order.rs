use diesel::{deserialize::Queryable, Selectable};
use serde::{Deserialize, Serialize};

use super::{
    customer::Customer,
    order_details::{NewOrderDetail, OrderDetails},
};

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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    #[serde(flatten)]
    pub order: Order,
    pub customer: Customer,
    pub order_items: Vec<OrderDetails>,
}

#[derive(Serialize)]
pub struct OR {
    pub orders: OrderN,
}

#[derive(Serialize, Queryable)]
pub struct OrderN {
    #[serde(rename = "id")]
    pub uuid: String,
    pub created_on: String,
    pub fulfilled_on: String,
    pub delivery_location: String,
    pub delivery_status: String,
    pub total_price: f64,
    pub customer: CustomerN,
    pub order_items: Vec<OrderItemsN>,
}

#[derive(Serialize, Queryable)]
pub struct CustomerN {
    #[serde(rename = "id")]
    pub uuid: String,
    pub first_name: String,
    pub last_name: String,
    pub phone_number: String,
}

#[derive(Serialize, Queryable)]
pub struct OrderItemsN {
    #[serde(rename = "id")]
    pub uuid: String,
    pub product: ProductN,
    pub quantity: f64,
    pub price: f64,
}

#[derive(Serialize, Queryable)]
pub struct ProductN {
    #[serde(rename = "id")]
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub image: String,
    pub price: f64,
    pub unit: String,
    pub category: CategoryN,
}

#[derive(Serialize, Queryable)]
pub struct CategoryN {
    pub uuid: String,
    pub name: String,
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
