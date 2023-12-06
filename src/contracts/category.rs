use diesel::{deserialize::Queryable, Selectable};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CategoryCreate {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::categories)]
pub struct Category {
    #[serde(rename = "id")]
    pub uuid: String,
    pub name: String,
}
