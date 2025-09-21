use diesel::prelude::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct ProductRating {
    #[serde(rename = "id")]
    pub uuid: String,
    pub first_name: String,
    pub last_name: String,
    pub rating: f64,
    pub review: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewProductRating {
    pub user_id: String,
    pub rating: f64,
    pub review: Option<String>,
}
