use diesel::deserialize::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable)]
pub struct InvoiceItem {
    pub uuid: String,
    pub product_id: String,
    pub product_name: String,
    pub invoice_id: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub discount_percent: f64,
    pub discount_amount: f64,
    pub total: f64,
}

#[derive(Deserialize)]
pub struct NewInvoiceItem {
    pub product_id: String,
    pub invoice_id: Option<String>,
    pub quantity: f64,
    pub unit_price: f64,
    pub discount_percent: Option<f64>,
    pub discount_amount: Option<f64>,
}
