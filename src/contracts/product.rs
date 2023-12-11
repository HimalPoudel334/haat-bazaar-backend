use diesel::deserialize::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    #[serde(rename = "id")]
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub image: String,
    pub price: f64,
    pub previous_price: f64,
    pub unit: String,
    pub unit_change: f64,
    pub stock: f64,
    pub category_id: String,
    pub category_name: String,
}

#[derive(Deserialize)]
pub struct ProductCreate {
    pub name: String,
    pub description: String,
    pub image: String,
    pub price: f64,
    pub previous_price: f64,
    pub unit: String,
    pub unit_change: f64,
    pub stock: f64,
    pub category_id: String,
}

#[derive(Deserialize)]
pub struct ProductStockUpdate {
    pub stock: f64,
}
