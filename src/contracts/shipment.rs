use diesel::prelude::Queryable;
use serde::{Deserialize, Serialize};

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
    first_name: Option<String>,
    last_name: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssingShipment {
    pub shipment_id: String,
    pub user_id: String,
}
