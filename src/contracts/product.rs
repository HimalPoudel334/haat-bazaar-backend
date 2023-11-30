use serde::{Deserialize, Serialize};

use super::category::Category;

#[derive(Serialize, Deserialize)]
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
    pub category: Category,
}
