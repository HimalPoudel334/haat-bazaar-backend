use diesel::prelude::Queryable;
use serde::Serialize;

#[derive(Serialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct Shipment {
    uuid: String,
    ship_date: String,
    address: String,
    city: String,
    state: String,
    country: String,
    zip_code: String,
    order_id: String,
    status: String,
    assigned_to: Option<String>,
}
