use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderDetails {
    pub uuid: String,
    pub quantity: f64,
    pub product_id: String,
    pub order_id: String,
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
