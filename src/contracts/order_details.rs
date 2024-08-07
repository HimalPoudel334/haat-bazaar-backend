use diesel::Queryable;
use serde::{Deserialize, Serialize};

use super::product::Product;

#[derive(Serialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct OrderDetails {
    #[serde(rename = "id")]
    pub uuid: String,
    pub product_id: String,
    pub order_id: String,
    pub quantity: f64,
    pub price: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderDetailsResponse {
    #[serde(rename = "id")]
    pub uuid: String,
    pub product: Product,
    pub order_id: String,
    pub quantity: f64,
    pub price: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewOrderDetail {
    pub quantity: f64,
    pub product_id: String,
    pub order_id: Option<String>,
    pub price: f64,
}
