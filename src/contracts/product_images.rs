use diesel::deserialize::Queryable;
use serde::Serialize;

#[derive(Serialize, Queryable)]
pub struct ProductImage {
    uuid: String,
    image_name: String,
}
