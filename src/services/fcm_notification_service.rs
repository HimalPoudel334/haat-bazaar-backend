use std::sync::Arc;

use async_trait::async_trait;
use fcm_service::{FcmMessage, FcmNotification, Target};

use crate::{
    db::connection::SqliteConnectionPool,
    models::admin_device::AdminDevice,
    services::notification_service::{NotificationEvent, NotificationService},
    utils::fcm_client::FcmClient,
};

use diesel::prelude::*;

pub struct FcmNotificationServiceImpl {
    fcm_client: Arc<FcmClient>,
    pool: SqliteConnectionPool,
}

impl FcmNotificationServiceImpl {
    pub fn new(fcm_client: Arc<FcmClient>, pool: SqliteConnectionPool) -> Self {
        Self { fcm_client, pool }
    }

    async fn get_admin_devices(&self) -> anyhow::Result<Vec<AdminDevice>> {
        use crate::schema::admin_devices::dsl::*;

        let conn = &mut self.pool.get()?;
        let devices = admin_devices
            .select(AdminDevice::as_select())
            .load::<AdminDevice>(conn)?;
        Ok(devices)
    }
}

#[async_trait]
impl NotificationService for FcmNotificationServiceImpl {
    async fn send_notification(&self, event: NotificationEvent) -> anyhow::Result<()> {
        let admin_dev = self.get_admin_devices().await?;

        if admin_dev.is_empty() {
            println!("No admin devices registered to send notification to.");
            return Ok(());
        }

        let (notification, data_payload) = match event {
            NotificationEvent::NewOrder(p) => {
                let mut notif = FcmNotification::new();
                notif.set_title(format!("New Order: {}", p.order_id));
                notif.set_body(format!(
                    "Customer: {}, Total: ${:.2}",
                    p.customer_name, p.total_amount
                ));
                let mut data = std::collections::HashMap::new();
                data.insert("order_id".to_string(), p.order_id);
                data.insert("customer_name".to_string(), p.customer_name);
                data.insert("total_amount".to_string(), p.total_amount.to_string());
                data.insert("event_type".to_string(), "new_order".to_string());
                (notif, data)
            }

            NotificationEvent::PaymentReceived(p) => {
                let mut notif = FcmNotification::new();
                notif.set_title("Payment Received!".to_string());
                notif.set_body(format!(
                    "Amount: ${:.2} for Order: {} through {}",
                    p.amount, p.order_id, p.payment_method
                ));
                let mut data = std::collections::HashMap::new();
                data.insert("event_type".to_string(), "payment_received".to_string());
                data.insert("order_id".to_string(), p.order_id);
                data.insert("amount".to_string(), p.amount.to_string());
                data.insert("transaction_id".to_string(), p.transaction_id);
                data.insert("payment_method".to_string(), p.payment_method);
                (notif, data)
            }

            NotificationEvent::OrderCancelled(p) => {
                let mut notif = FcmNotification::new();
                notif.set_title(format!("Order Cancelled: {}", p.order_id));

                let na = String::from("N/A");
                let body_reason = p.reason.as_ref().unwrap_or(&na);
                notif.set_body(format!("Reason: {}", body_reason));

                let mut data = std::collections::HashMap::new();
                data.insert("event_type".to_string(), "order_cancelled".to_string());
                data.insert("order_id".to_string(), p.order_id);

                if let Some(reason_val) = p.reason {
                    data.insert("reason".to_string(), reason_val);
                }
                (notif, data)
            }

            NotificationEvent::OrderFulfilled(p) => {
                let mut notif = FcmNotification::new();
                notif.set_title(format!("Order Fulfilled: {}", p.order_id));

                let details_for_body = p
                    .details
                    .as_ref()
                    .map_or("Your order has been completed.", |s| s.as_str());
                notif.set_body(format!("Details: {}", details_for_body));

                let mut data = std::collections::HashMap::new();
                data.insert("event_type".to_string(), "order_fulfilled".to_string());
                data.insert("order_id".to_string(), p.order_id);

                if let Some(details_val) = p.details {
                    data.insert("details".to_string(), details_val);
                }
                (notif, data)
            }

            NotificationEvent::Generic(p) => {
                let mut notif = FcmNotification::new();
                notif.set_title(p.title);
                notif.set_body(p.body);
                let mut data = p.data;

                if !data.contains_key("event_type") {
                    data.insert("event_type".to_string(), "generic_notification".to_string());
                }
                (notif, data)
            }
        };

        let mut tasks = vec![];
        for device in admin_dev {
            let mut fcm_message = FcmMessage::new();
            fcm_message.set_webpush(None);
            fcm_message.set_target(Target::Token(device.fcm_token.clone()));
            fcm_message.set_notification(Some(notification.clone()));
            fcm_message.set_data(Some(data_payload.clone()));

            let fcm_client_cloned = self.fcm_client.clone();

            let task = tokio::spawn(async move {
                let admin_id = device.user_id;
                match fcm_client_cloned.send_notification(fcm_message).await {
                    Ok(_) => println!(
                        "Successfully sent FCM notification to admin user: {}",
                        admin_id
                    ),
                    Err(e) => eprintln!(
                        "Failed to send FCM notification to admin user {}: {:?}",
                        admin_id, e
                    ),
                }
            });
            tasks.push(task);
        }

        for task in tasks {
            let _ = task.await;
        }

        Ok(())
    }
}
