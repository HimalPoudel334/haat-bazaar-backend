use serde::Deserialize;

#[derive(Deserialize)]
pub struct CategoryCreate {
    pub name: String,
}

pub struct Category {
    pub uuid: String,
    pub name: String,
}
