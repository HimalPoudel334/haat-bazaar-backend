use super::category::Category;

pub struct Product {
    pub name: String,
    pub description: String,
    pub image: String,
    pub price: f64,
    pub previous_price: f64,
    pub unit: String,
    pub unit_change: f64,
    pub stock: f64,
    pub category: Category,
}
