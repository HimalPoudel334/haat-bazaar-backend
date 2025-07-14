use regex::Regex;

pub struct PhoneNumber {
    phone_number: String,
}

impl PhoneNumber {
    pub fn from_str(number: String) -> Result<Self, &'static str> {
        const PHONE_NUMBER_REGEX: &str = r"^(?:\+?977)?9[78]\d{8}$";
        let phone_number_regex = Regex::new(PHONE_NUMBER_REGEX).unwrap();
        if phone_number_regex.is_match(number.as_str()) {
            Ok(Self {
                phone_number: number,
            })
        } else {
            Err("Invalid phone number")
        }
    }

    pub fn get_number(&self) -> String {
        self.phone_number.to_owned()
    }
}
