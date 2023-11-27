use diesel::prelude::*;

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
