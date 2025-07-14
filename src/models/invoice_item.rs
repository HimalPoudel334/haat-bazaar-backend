use diesel::prelude::*;
use uuid::Uuid;

use super::{invoice::Invoice, product::Product};

#[derive(Queryable, Selectable, Associations, Identifiable)]
#[diesel(table_name = crate::schema::invoice_items)]
#[diesel(belongs_to(Invoice))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct InvoiceItem {
    id: i32,
    uuid: String,
    quantity: f64,
    unit_price: f64,
    discount_percent: f64,
    discount_amount: f64,
    total: f64,

    invoice_id: i32,
    product_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::invoice_items)]
pub struct NewInvoiceItem {
    uuid: String,
    product_id: i32,
    invoice_id: i32,
    quantity: f64,
    unit_price: f64,
    discount_percent: f64,
    discount_amount: f64,
    total: f64,
}

impl NewInvoiceItem {
    pub fn new(
        product: &Product,
        invoice: &Invoice,
        quantity: f64,
        discount_percent: f64,
        discount_amount: f64,
    ) -> Self {
        let total = product.get_price() * quantity - discount_amount;
        Self {
            uuid: Uuid::new_v4().to_string(),
            product_id: product.get_id(),
            invoice_id: invoice.get_id(),
            quantity,
            unit_price: product.get_price(),
            discount_percent,
            discount_amount,
            total,
        }
    }
}
