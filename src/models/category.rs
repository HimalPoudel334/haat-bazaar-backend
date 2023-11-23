use serde::Serialize;
use uuid::Uuid;

pub struct Category {
    id: u32,
    uuid: String,
    name: String,
}

impl Category {
    pub fn get_id(&self) -> u32 {
        self.id
    }
    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Serialize)]
pub struct NewCategory {
    uuid: String,
    name: String,
}

impl NewCategory {
    pub fn new(name: String) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            name,
        }
    }
}
