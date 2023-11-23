use uuid::Uuid;

use super::category::Category;

pub struct Product {
    id: u32,
    uuid: String,
    name: String,
    description: String,
    image: String,
    price: f64,
    previous_price: f64,
    unit: String,
    unit_change: f64,
    stock: f64,
    category_id: u32,
}

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
    category_id: u32,
}

impl Product {
    pub fn get_id(&self) -> u32 {
        self.id
    }
    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }
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
        category: Category,
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
