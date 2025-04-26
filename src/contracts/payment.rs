use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Payment {
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

// #[derive(Serialize, Deserialize, Debug)]
// #[serde(rename_all = "camelCase")]
// pub struct EsewaTransactionResponse {
//     pub product_id: String,
//     pub product_name: String,
//     pub total_amount: String,
//     pub code: String,
//     pub merchant_name: String,
//     pub message: EsewaMessage,
//     pub transaction_details: EsewaTransactionDetails,
// }

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KhaltiQueryParams {
    pub pidx: String,
    pub txn_id: String,
    pub amount: String,
    pub total_amount: String,
    pub status: String,
    pub mobile: String,
    pub tidx: String,
    pub purchase_order_id: String,
    pub purchase_order_name: String,
    pub transaction_id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KhaltiPidxPayload {
    pub order_id: String,
}
