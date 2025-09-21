use super::{product::Product, user::User};
use diesel::prelude::*;

#[derive(Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(crate::models::user::User))]
#[diesel(belongs_to(crate::models::product::Product))]
#[diesel(table_name = crate::schema::product_ratings)]
pub struct ProductRating {
    id: i32,
    uuid: String,
    product_id: i32,
    user_id: i32,
    rating: f64,
    review: Option<String>,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}

impl ProductRating {
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_product_id(&self) -> i32 {
        self.product_id
    }

    pub fn get_user_id(&self) -> i32 {
        self.user_id
    }

    pub fn get_review(&self) -> Option<&str> {
        self.review.as_deref()
    }

    pub fn get_rating(&self) -> f64 {
        self.rating
    }

    pub fn get_created_at(&self) -> chrono::NaiveDateTime {
        self.created_at
    }

    pub fn get_updated_at(&self) -> chrono::NaiveDateTime {
        self.updated_at
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::product_ratings)]
pub struct NewProductRating {
    uuid: String,
    product_id: i32,
    user_id: i32,
    rating: f64,
    review: Option<String>,
}

impl NewProductRating {
    pub fn new(product: &Product, user: &User, rating: f64, review: Option<String>) -> Self {
        Self {
            uuid: uuid::Uuid::new_v4().to_string(),
            product_id: product.get_id(),
            user_id: user.get_id(),
            rating,
            review,
        }
    }
}
