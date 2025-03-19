use diesel::deserialize::Queryable;
use serde::Serialize;

#[derive(Serialize, Queryable, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductImage {
    #[serde(rename = "id")]
    uuid: String,
    image_name: String,
}
