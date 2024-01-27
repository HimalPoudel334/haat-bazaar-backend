use diesel::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Queryable)]
pub struct OrderDetails {
    #[serde(rename = "id")]
    pub uuid: String,
    pub product_id: String,
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
