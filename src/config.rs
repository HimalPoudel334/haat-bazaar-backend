#[derive(Debug, Clone)]
pub struct ApplicationConfiguration {
    pub server_address: String,
    pub server_port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expires_in: String,
    pub jwt_maxage: i32,
    pub refresh_token_secret: String,
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

        Self {
            server_address,
            server_port: server_port.parse::<u16>().unwrap(),
            database_url,
            jwt_secret,
            jwt_expires_in,
            jwt_maxage: jwt_maxage.parse::<i32>().unwrap(),
            refresh_token_secret,
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
        }
    }
}
