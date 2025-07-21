use serde::Deserialize;

#[derive(Deserialize)]
pub struct AdminDevice {
    pub user_id: String,
    pub fcm_token: String,
}
