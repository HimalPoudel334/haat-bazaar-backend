pub enum OrderStatus {
    PaymentPending,
    Pending,
    Processed,
    AwaitingDelivery,
    Fulfilled,
    Cancelled,
}

impl OrderStatus {
    pub fn value(&self) -> &str {
        match *self {
            OrderStatus::PaymentPending => "Payment Pending",
            OrderStatus::Pending => "Pending",
            OrderStatus::Processed => "Processed",
            OrderStatus::AwaitingDelivery => "Awaiting Delivery",
            OrderStatus::Fulfilled => "Fulfilled",
            OrderStatus::Cancelled => "Cancelled",
        }
    }

    pub fn from_str(string_value: &String) -> Result<Self, &str> {
        let normalized = string_value.trim().to_lowercase();

        match normalized.as_str() {
            "payment pending" => Ok(OrderStatus::PaymentPending),
            "pending" => Ok(OrderStatus::Pending),
            "processed" => Ok(OrderStatus::Processed),
            "awaiting delivery" => Ok(OrderStatus::AwaitingDelivery),
            "fulfilled" => Ok(OrderStatus::Fulfilled),
            "cancelled" => Ok(OrderStatus::Cancelled),
            _ => Err("Invalid order status. Valid values are: 'Payment Pending', 'Pending', 'Processed', 'Awaiting Delivery', 'Fulfilled', 'Cancelled'"),
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            OrderStatus::PaymentPending,
            OrderStatus::Pending,
            OrderStatus::Processed,
            OrderStatus::AwaitingDelivery,
            OrderStatus::Fulfilled,
            OrderStatus::Cancelled,
        ]
    }
}
