#[derive(Debug, Clone)]
pub struct ApplicationConfiguration {
    pub server_address: String,
    pub server_port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expires_in: String,
    pub jwt_maxage: i32,
    pub refresh_token_secret: String,
    pub refresh_token_maxage: i32,
    pub product_thumbnail_path: String,
    pub product_extraimages_path: String,
    pub esewa_merchant_id: String,
    pub esewa_merchant_secret: String,
    pub esewa_payment_verification_url: String,
    pub khalti_pidx_url: String,
    pub khalti_payment_verification_url: String,
    pub khalti_test_secret_key: String,
    pub khalti_test_public_key: String,
    pub khalti_live_secret_key: String,
    pub khalti_live_public_key: String,
    pub khalti_payment_confirm_callback_url: String,
    pub khalti_payment_confirm_callback_webiste_url: String,
    pub khalti_payment_confirm_lookup_url: String,
    pub firebase_service_account_key_path: String,
    pub firebase_project_id: String,
}

impl ApplicationConfiguration {
    pub fn init() -> Self {
        let server_address = std::env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS must be set");
        let server_port = std::env::var("SERVER_PORT").expect("SERVER_PORT must be set");
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let jwt_expires_in = std::env::var("JWT_EXPIRED_IN").expect("JWT_EXPIRED_IN must be set");
        let jwt_maxage = std::env::var("JWT_MAXAGE").expect("JWT_MAXAGE must be set");
        let refresh_token_secret =
            std::env::var("REFRESH_TOKEN_SECRET").expect("REFRESH_TOKEN_SECRET must be set");
        let refresh_token_maxage =
            std::env::var("REFRESH_TOKEN_MAXAGE").expect("REFRESH_TOKEN_MAXAGE must be set");
        let product_thumbnail_path =
            std::env::var("PRODUCT_THUMBNAIL_URL").expect("PRODUCT_THUMBNAIL_URL must be set");
        let product_extraimages_path =
            std::env::var("PRODUCT_EXTRA_IMAGE_URL").expect("PRODUCT_EXTRA_IMAGE_URL must be set");
        let esewa_merchant_id =
            std::env::var("ESEWA_MERCHANT_ID").expect("ESEWA_MERCHANT_ID must be set");
        let esewa_merchant_secret =
            std::env::var("ESEWA_MERCHANT_SECRET").expect("ESEWA_MERCHANT_SECRET must be set");
        let esewa_payment_verification_url = std::env::var("ESEWA_PAYMENT_VERIFICATION_URL")
            .expect("ESEWA_PAYMENT_VERIFICATION_URL must be set");
        let khalti_pidx_url =
            std::env::var("KHALTI_PIDX_URL").expect("KHALTI_PIDX_URL must be set");
        let khalti_payment_verification_url = std::env::var("KHALTI_PAYMENT_VERIFICATION_URL")
            .expect("KHALTI_PAYMENT_VERIFICATION_URL must be set");
        let khalti_test_secret_key =
            std::env::var("KHALTI_TEST_SECRET_KEY").expect("KHALTI_TEST_SECRET_KEY must be set");
        let khalti_test_public_key =
            std::env::var("KHALTI_TEST_PUBLIC_KEY").expect("KHALTI_TEST_PUBLIC_KEY must be set");
        let khalti_live_secret_key =
            std::env::var("KHALTI_LIVE_SECRET_KEY").expect("KHALTI_LIVE_SECRET_KEY must be set");
        let khalti_live_public_key =
            std::env::var("KHALTI_LIVE_PUBLIC_KEY").expect("KHALTI_LIVE_PUBLIC_KEY must be set");
        let khalti_payment_confirm_callback_url =
            std::env::var("KHALTI_PAYMENT_CONFIRM_CALLBACK_URL")
                .expect("KHALTI_PAYMENT_CONFIRM_CALLBACK_URL must be set");
        let khalti_payment_confirm_callback_webiste_url =
            std::env::var("KHALTI_PAYMENT_CONFIRM_CALLBACK_WEBSITE_URL")
                .expect("KHALTI_PAYMENT_CONFIRM_CALLBACK_WEBSITE_URL must be set");
        let khalti_payment_confirm_lookup_url = std::env::var("KHALTI_PAYMENT_CONFIRM_LOOKUP_URL")
            .expect("KHALTI_PAYMENT_CONFIRM_LOOKUP_URL must be set");
        let firebase_service_account_key_path = std::env::var("FIREBASE_SERVICE_ACCOUNT_KEY_PATH")
            .expect("FIREBASE_SERVICE_ACCOUNT_KEY_PATH must be set in .env file or environment");
        let firebase_project_id = std::env::var("FIREBASE_PROJECT_ID")
            .expect("FIREBASE_PROJECT_ID must be set in .env file or environment");

        Self {
            server_address,
            server_port: server_port.parse::<u16>().unwrap(),
            database_url,
            jwt_secret,
            jwt_expires_in,
            jwt_maxage: jwt_maxage.parse::<i32>().unwrap(),
            refresh_token_secret,
            refresh_token_maxage: refresh_token_maxage.parse::<i32>().unwrap(),
            product_thumbnail_path,
            product_extraimages_path,
            esewa_merchant_id,
            esewa_merchant_secret,
            esewa_payment_verification_url,
            khalti_pidx_url,
            khalti_payment_verification_url,
            khalti_test_secret_key,
            khalti_test_public_key,
            khalti_live_secret_key,
            khalti_live_public_key,
            khalti_payment_confirm_callback_url,
            khalti_payment_confirm_callback_webiste_url,
            khalti_payment_confirm_lookup_url,
            firebase_service_account_key_path,
            firebase_project_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmailConfiguration {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
    pub timeout_seconds: u64,
    pub use_tls: bool,
}

impl Default for EmailConfiguration {
    fn default() -> Self {
        Self {
            smtp_server: "smtp.gmail.com".to_string(),
            smtp_port: 587,
            username: "himalpou101@gmail.com".to_string(),
            password: "your-app-password".to_string(),
            from_email: "himalpou101@gmail.com".to_string(),
            from_name: "Haatbazar".to_string(),
            timeout_seconds: 30,
            use_tls: true,
        }
    }
}

impl EmailConfiguration {
    pub fn init() -> Self {
        let smtp_server = std::env::var("SMTP_SERVER").expect("SMTP_SERVER must be set");
        let smtp_port = std::env::var("SMTP_PORT").expect("SMTP_PORT must be set");
        let username = std::env::var("USERNAME").expect("USERNAME must be set");
        let password = std::env::var("PASSWORD").expect("PASSWORD must be set");
        let from_email = std::env::var("FROM_EMAIL").expect("FROM_EMAIL must be set");
        let from_name = std::env::var("FROM_NAME").expect("FROM_NAME must be set");
        let timeout_seconds =
            std::env::var("TIMEOUT_SECONDS").expect("TIMEOUT_SECONDS must be set");
        let use_tls = std::env::var("USE_TLS").expect("USE_TLS must be set");

        Self {
            smtp_server,
            smtp_port: smtp_port.parse::<u16>().unwrap(),
            username,
            password,
            from_email,
            from_name,
            timeout_seconds: timeout_seconds.parse::<u64>().unwrap(),
            use_tls: use_tls.parse::<bool>().unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompanyConfiguration {
    pub company_name: String,
    pub address: String,
    pub email: String,
    pub telephone: String,
    pub tax_rate: f64,
}

impl Default for CompanyConfiguration {
    fn default() -> Self {
        Self {
            company_name: "Haatbazaar".to_string(),
            address: "".to_string(),
            email: "info@haatbazaar.com.np".to_string(),
            telephone: "0234-56789".to_string(),
            tax_rate: 0.13,
        }
    }
}

impl CompanyConfiguration {
    pub fn init() -> Self {
        let company_name = std::env::var("COMPANY_NAME").expect("COMPANY_NAME must be set");
        let address = std::env::var("ADDRESS").expect("ADDRESS must be set");
        let email = std::env::var("EMAIL").expect("EMAIL must be set");
        let telephone = std::env::var("TELEPHONE").expect("TELEPHONE must be set");
        let tax_rate = std::env::var("TAX_RATE").expect("TAX_RATE must be set");

        Self {
            company_name,
            address,
            email,
            telephone,
            tax_rate: tax_rate.parse::<f64>().unwrap(),
        }
    }
}
