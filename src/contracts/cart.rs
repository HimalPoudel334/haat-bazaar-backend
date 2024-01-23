use diesel::Queryable;
use serde::Serialize;

#[derive(Serialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct Cart {
    #[serde(rename = "id")]
    uuid: String,
    product_id: String,
    customer_id: String,
    quantity: f64,
    created_on: String,
}
