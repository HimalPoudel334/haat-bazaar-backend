use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Identifiable, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::categories)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Category {
    id: i32,
    uuid: String,
    name: String,
}

impl Category {
    pub fn get_id(&self) -> i32 {
        self.id
    }
    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::categories)]
#[derive(Serialize)]
pub struct NewCategory {
    uuid: String,
    name: String,
}

impl NewCategory {
    pub fn new(name: String) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            name,
        }
    }
}
