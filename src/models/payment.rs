use diesel::prelude::*;
use uuid::Uuid;

use crate::base_types::payment_method::PaymentMethod;

use super::{customer::Customer, order::Order};

#[derive(Queryable, Selectable, Associations)]
#[diesel(table_name = crate::schema::payments)]
#[diesel(belongs_to(Customer))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Payment {
    id: i32,
    uuid: String,
    pay_date: String,
    amount: f64,
    payment_method: String,
    customer_id: i32,
    order_id: i32,
}

impl Payment {
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }

    pub fn get_pay_date(&self) -> &str {
        &self.pay_date
    }

    pub fn get_payment_method(&self) -> &str {
        &self.payment_method
    }

    pub fn get_amount(&self) -> f64 {
        self.amount
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::payments)]
pub struct NewPayment {
    uuid: String,
    pay_date: String,
    amount: f64,
    payment_method: String,
    customer_id: i32,
    order_id: i32,
}

impl NewPayment {
    pub fn new(
        payment_method: &PaymentMethod,
        customer: &Customer,
        order: &Order,
        pay_date: &String,
        amount: f64,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            payment_method: payment_method.value().to_owned(),
            customer_id: customer.get_id(),
            order_id: order.get_id(),
            pay_date: pay_date.to_owned(),
            amount,
        }
    }
}
