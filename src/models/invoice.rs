use uuid::Uuid;

use super::{customer::Customer, order::Order, payment::Payment};

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
