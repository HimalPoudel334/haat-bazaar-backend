use diesel::prelude::*;
use uuid::Uuid;

use super::{customer::Customer, order::Order, payment::Payment};

#[derive(Queryable, Selectable, Associations, Identifiable)]
#[diesel(table_name = crate::schema::invoices)]
#[diesel(belongs_to(Order))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Invoice {
    id: i32,
    uuid: String,
    invoice_number: i32,
    invoice_date: String,
    sub_total: f64,
    vat_percent: f64,
    vat_amount: f64,
    net_amount: f64,

    order_id: i32,
    customer_id: i32,
    payment_id: i32,
}

impl Invoice {
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub fn invoice_number(&self) -> i32 {
        self.invoice_number
    }

    pub fn sub_total(&self) -> f64 {
        self.sub_total
    }

    pub fn invoice_date(&self) -> &str {
        &self.invoice_date
    }

    pub fn vat_percent(&self) -> f64 {
        self.vat_percent
    }

    pub fn vat_amount(&self) -> f64 {
        self.vat_amount
    }

    pub fn net_amount(&self) -> f64 {
        self.net_amount
    }

    pub fn customer_id(&self) -> i32 {
        self.customer_id
    }

    pub fn order_id(&self) -> i32 {
        self.order_id
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::invoices)]
pub struct NewInvoice {
    uuid: String,
    invoice_number: i32,
    invoice_date: String,
    sub_total: f64,
    vat_percent: f64,
    vat_amount: f64,
    net_amount: f64,

    order_id: i32,
    customer_id: i32,
    payment_id: i32,
}

impl NewInvoice {
    pub fn new(
        invoice_date: &String,
        sub_total: f64,
        vat_percent: f64,
        order: &Order,
        customer: &Customer,
        payment: &Payment,
    ) -> Self {
        let vat_amount = sub_total * vat_percent / 100.0;
        let net_amount = sub_total + vat_amount;
        Self {
            uuid: Uuid::new_v4().to_string(),
            invoice_number: 1,
            sub_total,
            invoice_date: invoice_date.to_owned(),
            vat_percent,
            vat_amount,
            net_amount,
            customer_id: customer.get_id(),
            order_id: order.get_id(),
            payment_id: payment.get_id(),
        }
    }
}
