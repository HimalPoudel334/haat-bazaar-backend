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
