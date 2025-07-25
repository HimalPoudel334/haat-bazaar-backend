use async_trait::async_trait;

pub enum NotificationEvent {
    NewOrder(NewOrderPayload),
    PaymentReceived(PaymentReceivedPayload),
    OrderCancelled(OrderCancelledPayload),
    OrderFulfilled(OrderFulfilledPayload),
    Generic(GenericNotificationPayload),
}

pub struct NewOrderPayload {
    pub order_id: String,
    pub customer_name: String,
    pub total_amount: f64,
}

pub struct PaymentReceivedPayload {
    pub order_id: String,
    pub amount: f64,
    pub transaction_id: String,
    pub payment_method: String,
}

pub struct OrderCancelledPayload {
    pub order_id: String,
    pub reason: Option<String>,
}

pub struct OrderFulfilledPayload {
    pub order_id: String,
    pub details: Option<String>,
}

pub struct GenericNotificationPayload {
    pub title: String,
    pub body: String,
    pub data: std::collections::HashMap<String, String>,
}

#[async_trait]
pub trait NotificationService: Send + Sync + 'static {
    async fn send_notification(&self, event: NotificationEvent) -> Result<(), anyhow::Error>;
}
