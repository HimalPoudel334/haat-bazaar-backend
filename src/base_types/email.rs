use regex::Regex;

pub struct Email {
    email: String,
}

impl Email {
    pub fn from_str(e: String) -> Result<Self, &'static str> {
        const EMAIL_REGEX: &str = r"^[^\s@]+@[^\s@]+\.[^\s@]+$";
        let email_regex = Regex::new(EMAIL_REGEX).unwrap();
        if email_regex.is_match(e.as_str()) {
            Ok(Self { email: e })
        } else {
            Err("Invalid phone number")
        }
    }

    pub fn get_email(&self) -> String {
        self.email.to_owned()
    }
}
