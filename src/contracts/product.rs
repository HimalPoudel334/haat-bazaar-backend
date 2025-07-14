use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
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
pub struct ProductCreateB {
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

#[derive(MultipartForm)]
pub struct ProductCreate {
    pub name: Text<String>,
    pub description: Text<String>,
    pub image: Option<TempFile>,
    pub price: Text<f64>,
    #[multipart(rename = "previousPrice")]
    pub previous_price: Text<f64>,
    pub unit: Text<String>,
    #[multipart(rename = "unitChange")]
    pub unit_change: Text<f64>,
    pub stock: Text<f64>,
    #[multipart(rename = "categoryId")]
    pub category_id: Text<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryFilterParams {
    pub category_id: Option<String>,
}
