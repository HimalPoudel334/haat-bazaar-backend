pub struct User {
    id: i32,
    uuid: String,
    first_name: String,
    last_name: String,
    phone_number: String,
    email: String,
    password: String,
    user_type: String,
}

impl User {
    pub const USERTYPE_ADMIN: &'static str = "Admin";
    pub const USERTYPE_CUSTOMER: &'static str = "Customer";

    pub fn new(
        id: i32,
        uuid: String,
        first_name: String,
        last_name: String,
        phone_number: String,
        email: String,
        password: String,
    ) -> Self {
        Self {
            id,
            uuid,
            first_name,
            last_name,
            phone_number,
            email,
            password,
            user_type : Self::USERTYPE_CUSTOMER.to_string(),
        }
    }
}