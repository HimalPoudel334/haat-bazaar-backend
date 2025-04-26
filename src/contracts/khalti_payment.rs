use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct UserInfo {
    pub name: String,
    pub email: String,
    pub phone: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AmountBreakdown {
    label: String,
    amount: i64, // in paise
}

impl AmountBreakdown {
    pub fn new(label: String, amount: f64) -> Self {
        Self {
            label,
            amount: (amount * 100.0) as i64,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProductDetail {
    identity: String,
    name: String,
    total_price: i64, // in paise
    quantity: f64,
    unit_price: i64, // in paise
}

impl ProductDetail {
    pub fn new(
        identity: String,
        name: String,
        total_price: f64,
        unit_price: f64,
        quantity: f64,
    ) -> Self {
        ProductDetail {
            identity,
            name,
            total_price: (total_price * 100.0) as i64,
            unit_price: (unit_price * 100.0) as i64,
            quantity,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KhaltiPaymentPayload {
    return_url: String,
    website_url: String,
    amount: i64, // in paise
    purchase_order_id: String,
    purchase_order_name: String,
    customer_info: UserInfo,
    amount_breakdown: Option<Vec<AmountBreakdown>>,
    product_details: Option<Vec<ProductDetail>>,
    merchant_username: String,
    merchant_extra: String,
}

impl KhaltiPaymentPayload {
    pub fn create(
        return_url: String,
        website_url: String,
        amount: f64, // in rupees
        purchase_order_id: String,
        purchase_order_name: String,
        customer_info: UserInfo,
        amount_breakdown: Option<Vec<AmountBreakdown>>,
        product_details: Option<Vec<ProductDetail>>,
        merchant_username: String,
        merchant_extra: String,
    ) -> Self {

        Self {
            return_url,
            website_url,
            amount: (amount * 100.0) as i64,
            purchase_order_id,
            purchase_order_name,
            customer_info,
            amount_breakdown,
            product_details,
            merchant_username,
            merchant_extra,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct KhaltiResponse {
    pub pidx: String,
    pub payment_url: String,
    pub expires_at: String,
    pub expires_in: i32,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KhaltiResponseCamelCase {
    pub pidx: String,
    pub payment_url: String,
    pub expires_at: String,
    pub expires_in: i32,
}

impl From<KhaltiResponse> for KhaltiResponseCamelCase {
    fn from(api_response: KhaltiResponse) -> Self {
        Self {
            pidx: api_response.pidx,
            payment_url: api_response.payment_url,
            expires_at: api_response.expires_at,
            expires_in: api_response.expires_in,
        }
    }
}
