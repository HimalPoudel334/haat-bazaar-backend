use diesel::prelude::*;
use uuid::Uuid;

use crate::models::user::User;

#[derive(Default, PartialEq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::refresh_tokens)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct RefreshToken {
    id: i32,
    uuid: String,
    token: String,
    user_id: i32,
    expires_on: String,
}

impl RefreshToken {
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }

    pub fn get_user_id(&self) -> i32 {
        self.user_id
    }

    pub fn get_token(&self) -> &str {
        &self.token
    }

    pub fn get_expires_on(&self) -> &str {
        &self.expires_on
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::refresh_tokens)]
pub struct NewRefreshToken {
    uuid: String,
    token: String,
    user_id: i32,
    expires_on: String,
}

impl NewRefreshToken {
    pub fn new(user: &User, token: String, expires_on: &String) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            token,
            user_id: user.get_id(),
            expires_on: expires_on.to_owned(),
        }
    }
}
