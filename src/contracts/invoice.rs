use diesel::deserialize::Queryable;
use serde::{Deserialize, Serialize};

use super::invoice_item::{InvoiceItem, NewInvoiceItem};

#[derive(Serialize, Queryable)]
pub struct Invoice {
    pub uuid: String,
    pub invoice_number: i32,
    pub invoice_date: String,
    pub customer_name: String,
    pub sub_total: f64,
    pub vat_percent: f64,
    pub vat_amount: f64,
    pub net_amount: f64,

    pub order_id: String,
    pub user_id: String,
    pub payment_id: String,
    pub invoice_items: Vec<InvoiceItem>,
}

// This struct is used for displaying all the invoices only
#[derive(Serialize, Queryable)]
pub struct InvoiceOnly {
    pub uuid: String,
    pub invoice_number: i32,
    pub invoice_date: String,
    pub customer_name: String,
    pub sub_total: f64,
    pub vat_percent: f64,
    pub vat_amount: f64,
    pub net_amount: f64,

    pub order_id: String,
    pub user_id: String,
    pub payment_id: String,
}

#[derive(Deserialize)]
pub struct NewInvoice {
    pub invoice_date: String,
    pub sub_total: f64,
    pub vat_percent: f64,

    pub order_id: String,
    pub user_id: String,
    pub payment_id: String,
    pub invoice_items: Vec<NewInvoiceItem>,
}
