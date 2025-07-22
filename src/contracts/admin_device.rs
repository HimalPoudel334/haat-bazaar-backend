use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminDevice {
    pub user_id: String,
    pub fcm_token: String,
}
