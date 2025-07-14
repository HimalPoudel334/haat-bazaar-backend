use diesel::Queryable;
use serde::{Deserialize, Serialize};

use super::product::Product;

#[derive(Serialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct OrderItems {
    #[serde(rename = "id")]
    pub uuid: String,
    pub product_id: String,
    pub order_id: String,
    pub quantity: f64,
    pub price: f64,
    pub amount: f64,
    pub discount: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderItemResponse {
    #[serde(rename = "id")]
    pub uuid: String,
    pub product: Product,
    pub order_id: String,
    pub quantity: f64,
    pub price: f64,
    pub discount: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewOrderItem {
    pub quantity: f64,
    pub product_id: String,
    pub order_id: Option<String>,
    pub price: f64,
    pub discount: f64,
}
