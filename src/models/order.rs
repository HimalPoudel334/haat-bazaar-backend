pub struct Order {
    id: u32,
    uuid: String,
    order_date: String,
    fulfilled_on: String,
    delivery_location: String,
    delivery_status: String,
    total_price: f64,
    customer_id: u32,
}

impl Order {
    pub fn get_id(&self) -> u32 {
        self.id
    }
    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }
}
