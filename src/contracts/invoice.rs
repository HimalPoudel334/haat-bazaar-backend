use serde::Serialize;

use super::invoice_item::{InvoiceItem, NewInvoiceItem};

#[derive(Serialize)]
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
    pub customer_id: String,
    pub payment_id: String,
    pub invoice_items: Vec<InvoiceItem>,
}

pub struct NewInvoice {
    pub invoice_date: String,
    pub sub_total: f64,
    pub vat_percent: f64,

    pub order_id: String,
    pub customer_id: String,
    pub payment_id: String,
    pub invoice_items: Vec<NewInvoiceItem>,
}
