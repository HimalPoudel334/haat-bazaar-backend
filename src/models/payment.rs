use diesel::prelude::*;
use uuid::Uuid;

use crate::base_types::payment_method::PaymentMethod;

use super::customer::Customer;

#[derive(Queryable, Selectable, Associations)]
#[diesel(table_name = crate::schema::payments)]
#[diesel(belongs_to(Customer))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Payment {
    id: i32,
    uuid: String,
    payment_method: String,
    customer_id: i32,
    pay_date: String,
    amount: f64,
}

pub struct NewPayment {
    uuid: String,
    payment_method: String,
    customer_id: i32,
    pay_date: String,
    amount: f64,
}

impl NewPayment {
    pub fn new(
        payment_method: &PaymentMethod,
        customer: &Customer,
        pay_date: &String,
        amount: f64,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            payment_method: payment_method.value().to_owned(),
            customer_id: customer.get_id(),
            pay_date: pay_date.to_owned(),
            amount,
        }
    }
}
