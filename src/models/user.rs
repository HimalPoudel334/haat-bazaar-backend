use argon2::password_hash::Error;
use diesel::{
    prelude::{Identifiable, Insertable, Queryable},
    Selectable,
};
use uuid::Uuid;

use crate::{base_types::phone_number::PhoneNumber, utils::password_helper::hash_password};

#[derive(Default, PartialEq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    id: i32,
    uuid: String,
    first_name: String,
    last_name: String,
    phone_number: String,
    email: String,
    password: String,
    user_type: String,
    location: Option<String>,
    nearest_landmark: Option<String>,
}

impl User {
    pub const USERTYPE_ADMIN: &'static str = "Admin";
    pub const USERTYPE_CUSTOMER: &'static str = "Customer";
    pub const USERTYPE_DELIVERY: &'static str = "Delivery";

    pub fn new(
        id: i32,
        uuid: String,
        first_name: String,
        last_name: String,
        phone_number: String,
        email: String,
        password: String,
        location: Option<String>,
        nearest_landmark: Option<String>,
    ) -> Self {
        Self {
            id,
            uuid,
            first_name,
            last_name,
            phone_number,
            email,
            password,
            user_type: Self::USERTYPE_CUSTOMER.to_string(),
            location,
            nearest_landmark,
        }
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }

    pub fn get_first_name(&self) -> &str {
        &self.first_name
    }

    pub fn get_last_name(&self) -> &str {
        &self.last_name
    }

    pub fn get_full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    pub fn get_phone_number(&self) -> &str {
        &self.phone_number
    }

    pub fn get_email(&self) -> &str {
        &self.email
    }

    pub fn get_password(&self) -> &str {
        &self.password
    }

    pub fn is_admin(&self) -> bool {
        self.user_type == Self::USERTYPE_ADMIN
    }

    pub fn get_user_type(&self) -> &str {
        &self.user_type
    }

    pub fn get_location(&self) -> Option<&str> {
        self.location.as_deref()
    }

    pub fn get_nearest_landmark(&self) -> Option<&str> {
        self.nearest_landmark.as_deref()
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]

pub struct NewUser {
    uuid: String,
    first_name: String,
    last_name: String,
    phone_number: String,
    email: String,
    password: String,
    user_type: String,
    location: Option<String>,
    nearest_landmark: Option<String>,
}

impl NewUser {
    pub const USERTYPE_ADMIN: &'static str = "Admin";
    pub const USERTYPE_CUSTOMER: &'static str = "Customer";
    pub const USERTYPE_DELIVERY: &'static str = "Delivery";

    pub fn new(
        first_name: String,
        last_name: String,
        phone_number: PhoneNumber,
        email: String,
        password: String,
        location: Option<String>,
        nearest_landmark: Option<String>,
    ) -> Result<Self, Error> {
        let hashed_password = hash_password(&password)?; // Propagate the error
        Ok(Self {
            uuid: Uuid::new_v4().to_string(),
            first_name,
            last_name,
            phone_number: phone_number.get_number(),
            email,
            password: hashed_password,
            user_type: Self::USERTYPE_CUSTOMER.to_string(),
            location,
            nearest_landmark,
        })
    }
}
