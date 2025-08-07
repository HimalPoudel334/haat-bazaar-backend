use diesel::prelude::QueryableByName;
use diesel::sql_types::{Integer, Nullable, Text};
use diesel::sqlite::Sqlite;
use diesel::{deserialize::Queryable, Selectable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)] //for selection to vm
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "id")]
    pub uuid: String,
    pub first_name: String,
    pub last_name: String,
    pub phone_number: String,
    pub email: String,
    pub user_type: String,
    pub location: Option<String>,
    pub nearest_landmark: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserCreate {
    pub first_name: String,
    pub last_name: String,
    pub phone_number: String,
    pub email: String,
    pub password: String,
    pub location: Option<String>,
    pub nearest_landmark: Option<String>,
}

#[derive(QueryableByName, Serialize)]
#[diesel(check_for_backend(Sqlite))]
#[serde(rename_all = "camelCase")]
pub struct UserPendingShipments {
    #[diesel(sql_type = Text)]
    #[serde(rename = "id")]
    pub user_id: String,

    #[diesel(sql_type = Text)]
    pub full_name: String,

    #[diesel(sql_type = Nullable<Text>)]
    pub location: Option<String>,

    #[diesel(sql_type = Nullable<Text>)]
    pub nearest_landmark: Option<String>,

    #[diesel(sql_type = Integer)]
    pub pending_shipment_count: i32,
}
