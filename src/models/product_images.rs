use diesel::prelude::*;

use super::product::Product;

#[derive(Queryable, Selectable, Associations, Identifiable)]
#[diesel(table_name = crate::schema::product_images)]
#[diesel(belongs_to(Product))]
pub struct ProductImages {
    id: i32,
    uuid: String,
    image_name: String,
    product_id: i32,
}

impl ProductImages {
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }

    pub fn get_name(&self) -> &str {
        &self.image_name
    }

    pub fn get_product_id(&self) -> i32 {
        self.product_id
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::product_images)]
pub struct NewProductImage {
    uuid: String,
    image_name: String,
    product_id: i32,
}

impl NewProductImage {
    pub fn new(image_name: &String, product: &Product) -> Self {
        Self {
            uuid: uuid::Uuid::new_v4().to_string(),
            image_name: image_name.to_owned(),
            product_id: product.get_id(),
        }
    }
}
