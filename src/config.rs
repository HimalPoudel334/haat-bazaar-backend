#[derive(Debug, Clone)]
pub struct ApplicationConfiguration {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expires_in: String,
    pub jwt_maxage: i32,
    pub product_thumbnail_path: String,
    pub product_extraimages_path: String,
}

impl ApplicationConfiguration {
    pub fn init() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let jwt_expires_in = std::env::var("JWT_EXPIRED_IN").expect("JWT_EXPIRED_IN must be set");
        let jwt_maxage = std::env::var("JWT_MAXAGE").expect("JWT_MAXAGE must be set");
        let product_thumbnail_path =
            std::env::var("PRODUCT_THUMBNAIL_URL").expect("PRODUCT_THUMBNAIL_URL must be set");
        let product_extraimages_path =
            std::env::var("PRODUCT_EXTRA_IMAGE_URL").expect("PRODUCT_EXTRA_IMAGE_URL must be set");

        Self {
            database_url,
            jwt_secret,
            jwt_expires_in,
            jwt_maxage: jwt_maxage.parse::<i32>().unwrap(),
            product_thumbnail_path,
            product_extraimages_path,
        }
    }
}
