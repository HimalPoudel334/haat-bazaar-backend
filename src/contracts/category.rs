use diesel::deserialize::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CategoryCreate {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct Category {
    pub uuid: String,
    pub name: String,
}
