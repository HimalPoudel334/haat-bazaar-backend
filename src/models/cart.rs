use chrono::NaiveDateTime;

pub struct Cart {
    id: i32,
    uuid: String,
    product_id: i32,
    customer_id: i32,
    quantity: f64,
    created_on: NaiveDateTime,
}
