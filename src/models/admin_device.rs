use diesel::{
    prelude::{Insertable, Queryable},
    Selectable,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::user::User;

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::admin_devices)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct AdminDevice {
    pub id: i32,
    pub uuid: String,
    pub user_id: i32,
    pub fcm_token: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::admin_devices)]
pub struct NewAdminDevice {
    pub user_id: i32,
    pub fcm_token: String,
    pub uuid: String,
}

impl NewAdminDevice {
    pub fn new(user: &User, fcm_token: String) -> Self {
        Self {
            user_id: user.get_id(),
            fcm_token,
            uuid: Uuid::new_v4().to_string(),
        }
    }
}
