use diesel::prelude::*;
use uuid::Uuid;

use super::category::Category;

#[derive(Queryable, Selectable, Associations, Identifiable)]
#[diesel(table_name = crate::schema::products)]
#[diesel(belongs_to(Category))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Product {
    id: i32,
    uuid: String,
    name: String,
    description: String,
    image: String,
    price: f64,
    previous_price: f64,
    unit: String,
    unit_change: f64,
    stock: f64,
    category_id: i32,
}

impl Product {
    pub fn get_id(&self) -> i32 {
        self.id
    }
    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::products)]
pub struct NewProduct {
    uuid: String,
    name: String,
    description: String,
    image: String,
    price: f64,
    previous_price: f64,
    unit: String,
    unit_change: f64,
    stock: f64,
    category_id: i32,
}

impl NewProduct {
    pub fn new(
        name: String,
        description: String,
        image: String,
        price: f64,
        previous_price: f64,
        unit: String,
        unit_change: f64,
        stock: f64,
        category: &Category,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            name,
            description,
            image,
            price,
            previous_price,
            unit,
            unit_change,
            stock,
            category_id: category.get_id(),
        }
    }
}
