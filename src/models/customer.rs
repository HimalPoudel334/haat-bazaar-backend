use uuid::Uuid;

use crate::base_types::phone_number::PhoneNumber;

pub struct Customer {
    id: u32,
    uuid: String,
    first_name: String,
    last_name: String,
    phone_number: String,
    password: String,
}

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
