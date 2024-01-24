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

    pub fn get_category_id(&self) -> i32 {
        self.category_id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_price(&self) -> f64 {
        self.price
    }

    pub fn get_stock(&self) -> f64 {
        self.stock
    }

    //oh lord have mercy
    //I have created a dependency of this model to contracts which should be avoided
    pub fn as_response(&self, category: &Category) -> crate::contracts::product::Product {
        crate::contracts::product::Product {
            uuid: self.uuid.to_owned(),
            name: self.name.to_owned(),
            description: self.description.to_owned(),
            image: self.image.to_owned(),
            price: self.price,
            previous_price: self.previous_price,
            unit: self.unit.to_owned(),
            unit_change: self.unit_change,
            stock: self.stock,
            category_id: category.get_uuid().to_string(),
            category_name: category.get_name().to_string(),
        }
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
