use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use diesel::deserialize::Queryable;
use serde::{Deserialize, Serialize};

use crate::contracts::category::Category;

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
    pub category: Category,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(MultipartForm)]
pub struct UploadForm {
    pub images: Vec<TempFile>,
    #[multipart(rename = "thumbnail")]
    pub image: Option<TempFile>,
}
