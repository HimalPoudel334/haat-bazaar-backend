use serde::{Deserialize, Serialize};

/*#[derive(Debug, Deserialize, Serialize)]
pub struct KhaltiPayment {
    pub return_url: String,
    pub website_url: String,
    pub amount: i64,
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
}*/

#[derive(Debug, Deserialize, Serialize)]
pub struct CustomerInfo {
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
pub struct KhaltiPayment {
    return_url: String,
    website_url: String,
    amount: i64, // in paise
    purchase_order_id: String,
    purchase_order_name: String,
    customer_info: CustomerInfo,
    amount_breakdown: Option<Vec<AmountBreakdown>>,
    product_details: Option<Vec<ProductDetail>>,
    merchant_username: String,
    merchant_extra: String,
}

impl KhaltiPayment {
    pub fn init(
        return_url: String,
        website_url: String,
        amount: f64, // in rupees
        purchase_order_id: String,
        purchase_order_name: String,
        customer_info: CustomerInfo,
        amount_breakdown: Option<Vec<AmountBreakdown>>,
        product_details: Option<Vec<ProductDetail>>,
        merchant_username: String,
        merchant_extra: String,
    ) -> Self {
        // let amount_breakdown = amount_breakdown.map(|breakdown| {
        //     breakdown.into_iter().map(|ab| AmountBreakdown {
        //         label: ab.label,
        //         amount: (ab.amount * 100.0) as i64,
        //     }).collect()
        // });

        // let product_details = product_details.map(|details| {
        //     details.into_iter().map(|pd| ProductDetail {
        //         identity: pd.identity,
        //         name: pd.name,
        //         total_price: (pd.total_price * 100.0) as i64,
        //         unit_price: (pd.unit_price * 100.0) as i64,
        //         quantity: pd.quantity,
        //     }).collect()
        // });

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
