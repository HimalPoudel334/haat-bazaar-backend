use diesel::prelude::*;
use uuid::Uuid;

use crate::base_types::phone_number::PhoneNumber;

#[derive(Default, PartialEq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::customers)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Customer {
    id: i32,
    uuid: String,
    first_name: String,
    last_name: String,
    phone_number: String,
    password: String,
}

impl Customer {
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_uuid(&self) -> &String {
        &self.uuid
    }

    pub fn get_first_name(&self) -> &String {
        &self.first_name
    }

    pub fn get_last_name(&self) -> &String {
        &self.last_name
    }

    pub fn get_phone_number(&self) -> &String {
        &self.phone_number
    }

    pub fn get_password(&self) -> &String {
        &self.password
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::customers)]
pub struct NewCustomer {
    uuid: String,
    first_name: String,
    last_name: String,
    phone_number: String,
    password: String,
}

impl NewCustomer {
    pub fn new(
        first_name: String,
        last_name: String,
        phone_number: PhoneNumber,
        password: String,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            first_name,
            last_name,
            phone_number: phone_number.get_number(),
            password,
        }
    }
}
