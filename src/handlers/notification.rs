use std::sync::Arc;

use actix_web::{http::StatusCode, post, web, HttpResponse};
use diesel::prelude::*;
use fcm_service::{FcmMessage, FcmNotification, Target};

use crate::{
    contracts::order::OrderCreatedPayload,
    db::connection::{get_conn, SqliteConnectionPool},
    models::admin_device::AdminDevice,
    utils::fcm_client::FcmClient,
};

#[post("/order-created")]
pub async fn new_order_created(
    fcm_client: web::Data<Arc<FcmClient>>,
    payload: web::Json<OrderCreatedPayload>,
    pool: web::Data<SqliteConnectionPool>,
) -> HttpResponse {
    use crate::schema::admin_devices::dsl::*;

    let conn = &mut get_conn(&pool);

    let admin_dev = match admin_devices
        .select(AdminDevice::as_select())
        .load::<AdminDevice>(conn)
    {
        Ok(d) => d,
        Err(_) => return HttpResponse::Ok().status(StatusCode::OK).json(
            serde_json::json!({"message": "Order created but error while getting admin devices"}),
        ),
    };

    if admin_dev.is_empty() {
        println!("No admin devices registered to send notification to.");
        return HttpResponse::Ok().body("No admin devices registered, notification not sent.");
    }

    // --- 2. Construct FCM Message using fcm-service types ---
    let mut notification = FcmNotification::new();
    notification.set_title(format!("New Order: {}", payload.order_id));
    notification.set_body(format!(
        "Customer: {}, Total: ${:.2}",
        payload.customer_name, payload.total_amount
    ));
    notification.set_image(None);

    let mut data_payload = std::collections::HashMap::new();
    data_payload.insert("order_id".to_string(), payload.order_id.clone());
    data_payload.insert("customer_name".to_string(), payload.customer_name.clone());
    data_payload.insert("total_amount".to_string(), payload.total_amount.to_string());
    data_payload.insert("event_type".to_string(), "new_order".to_string());

    // --- 3. Send Notification to each Admin Device ---
    let mut tasks = vec![];
    for device in admin_dev {
        let mut fcm_message = FcmMessage::new();
        fcm_message.set_webpush(None);
        fcm_message.set_target(Target::Token(device.fcm_token.clone()));
        fcm_message.set_notification(Some(notification.clone()));
        fcm_message.set_data(Some(data_payload.clone()));

        let fcm_client_cloned = fcm_client.clone();

        let task = tokio::spawn(async move {
            let admin_id = device.user_id;
            match fcm_client_cloned.send_notification(fcm_message).await {
                Ok(_) => println!(
                    "Successfully sent FCM notification to admin user: {}",
                    admin_id
                ),
                Err(e) => println!(
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

    HttpResponse::Ok().body("New order received and notifications dispatched!")
}
