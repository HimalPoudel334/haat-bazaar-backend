use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct KhaltiPayment {
    pub return_url: String,
    pub website_url: String,
    pub amount: String,
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
    pub amount: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProductDetail {
    pub identity: String,
    pub name: String,
    pub total_price: String,
    pub quantity: i32,
    pub unit_price: String,
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
