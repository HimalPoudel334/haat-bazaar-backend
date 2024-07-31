use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct KhaltiPayment {
    pub return_url: String,
    pub website_url: String,
    pub amount: f64,
    pub purchase_order_id: String,
    pub purchase_order_name: String,
    pub customer_info: CustomerInfo,
    pub amount_breakdown: Option<Vec<AmountBreakdown>>,
    pub product_details: Option<Vec<ProductDetail>>,
    pub merchant_username: String,
    pub merchant_extra: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CustomerInfo {
    pub name: String,
    pub email: String,
    pub phone: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AmountBreakdown {
    pub label: String,
    pub amount: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProductDetail {
    pub identity: String,
    pub name: String,
    pub total_price: f64,
    pub quantity: i32,
    pub unit_price: f64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct KhaltiResponse {
    pub pidx: String,
    pub payment_url: String,
    pub expires_at: String,
    pub expires_in: i32,
}
