use diesel::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct Cart {
    #[serde(rename = "id")]
    pub uuid: String,
    pub product_id: String, 
    pub product_name: String,
    pub quantity: f64,
    pub sku: String,
    pub created_on: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCart {
    pub product_id: String,
    pub quantity: f64,
    pub created_on: String,
}

#[derive(Deserialize)]
pub struct UpdateCartQuantity {
    pub new_quantity: f64,
}
