use diesel::{
    prelude::{Insertable, Queryable},
    Selectable,
};
use serde::{Deserialize, Serialize};

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
}
