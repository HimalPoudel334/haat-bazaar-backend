use anyhow::Result;

use fcm_service::{FcmMessage, FcmService};

pub struct FcmClient {
    fcm_service: FcmService,
}

impl FcmClient {
    pub async fn new(service_account_key_path: &str) -> Result<Self> {
        let fcm_service_instance = FcmService::new(service_account_key_path);

        Ok(Self {
            fcm_service: fcm_service_instance,
        })
    }

    pub async fn send_notification(&self, message: FcmMessage) -> Result<()> {
        match self.fcm_service.send_notification(message).await {
            Ok(_) => {
                println!("FCM: Message sent successfully.");
                Ok(())
            }
            Err(e) => {
                eprintln!("FCM: Failed to send message. Error: {:?}", e);
                Err(anyhow::anyhow!("FCM message sending failed: {}", e))
            }
        }
    }
}
