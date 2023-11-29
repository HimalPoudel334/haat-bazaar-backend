use diesel::{deserialize::Queryable, Selectable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::customers)]
pub struct Customer {
    pub uuid: String,
    pub first_name: String,
    pub last_name: String,
    pub phone_number: String,
}

#[derive(Deserialize)]
pub struct CustomerCreate {
    pub first_name: String,
    pub last_name: String,
    pub phone_number: String,
    pub password: String,
}
