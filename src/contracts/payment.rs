use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Payment {
    pub uuid: String,
    pub payment_method: String,
    pub customer_id: String,
    pub order_id: String,
    pub pay_date: String,
    pub amount: f64,
}

#[derive(Deserialize)]
pub struct NewPayment {
    pub payment_method: String,
    pub customer_id: String,
    pub order_id: String,
    pub pay_date: String,
    pub amount: f64,
}
