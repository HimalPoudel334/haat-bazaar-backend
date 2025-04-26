use diesel::prelude::*;
use uuid::Uuid;

use crate::base_types::payment_method::PaymentMethod;

use super::{user::User, order::Order};

#[derive(Queryable, Selectable, Associations, Identifiable)]
#[diesel(table_name = crate::schema::payments)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct  Payment {
    id: i32,
    uuid: String,
    pay_date: String,
    amount: f64,
    payment_method: String,
    user_id: i32,
    order_id: i32,
    transaction_id: String,
    tendered: f64,
    change: f64,
    discount: f64
}

impl Payment {
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }

    pub fn get_transaction_id(&self) -> &str {
        &self.transaction_id
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

    pub fn get_user_id(&self) -> i32 {
        self.user_id
    }

    pub fn get_order_id(&self) -> i32 {
        self.order_id
    }

    pub fn get_tendered(&self) -> f64 {
        self.tendered
    }

    pub fn get_change(&self) -> f64 {
        self.change
    }

    pub fn get_discount(&self) -> f64 {
        self.discount
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::payments)]
pub struct NewPayment {
    uuid: String,
    pay_date: String,
    amount: f64,
    payment_method: String,
    user_id: i32,
    order_id: i32,
    transaction_id: String,
    tendered: f64,
    change: f64,
    discount: f64
}

impl NewPayment {
    pub fn new(
        payment_method: &PaymentMethod,
        transaction_id: &String,
        user: &User,
        order: &Order,
        amount: f64,
        tendered: f64,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            transaction_id: transaction_id.to_owned(),
            payment_method: payment_method.value().to_owned(),
            user_id: user.get_id(),
            order_id: order.get_id(),
            pay_date: chrono::Local::now().to_rfc3339(),
            amount,
            tendered,
            change: tendered - amount,
            discount: 0.0
        }
    }
}
