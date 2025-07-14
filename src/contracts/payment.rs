use diesel::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Queryable)]
#[diesel(table_name = crate::schema::payments)]
#[serde(rename_all = "camelCase")]
pub struct Payment {
    #[serde(rename = "id")]
    pub uuid: String,
    pub payment_method: String,
    pub user_id: String,
    pub order_id: String,
    pub pay_date: String,
    pub amount: f64,
    pub tendered: f64,
    pub change: f64,
    pub discount: f64,
    pub transaction_id: String,
    pub status: String,
    pub service_charge: f64,
    pub refunded: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewPayment {
    pub payment_method: String,
    pub user_id: String,
    pub order_id: String,
    pub amount: f64,
    pub tendered: f64,
    pub transaction_id: Option<String>,
    pub status: String,
}

// for esewa payment
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EsewaMessage {
    pub success_message: String,
    pub technical_success_message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EsewaTransactionDetails {
    pub status: String,
    pub reference_id: String,
    pub date: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EsewaCallbackResponse {
    pub product_id: String,
    pub product_name: String,
    pub total_amount: String,
    pub environment: Option<String>,
    pub code: Option<String>,
    pub merchant_name: String,
    pub message: EsewaMessage,
    pub transaction_details: EsewaTransactionDetails,
}

#[derive(Deserialize, Debug)]
pub struct KhaltiPaymentLookupResponse {
    pub pidx: String,
    pub total_amount: f64,
    pub status: String,
    pub transaction_id: Option<String>,
    pub fee: f64,
    pub refunded: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KhaltiPidxPayload {
    pub order_id: String,
}

#[derive(Deserialize, Debug)]
pub struct KhaltiPaymentConfirmPayload {
    pub pidx: String,
    pub order_id: String,
}
