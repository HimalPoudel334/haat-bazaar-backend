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
        match string_value.as_str(){
            "Payment Pending" => Ok(OrderStatus::PaymentPending),
            "Pending" => Ok(OrderStatus::Pending),
            "Processed" => Ok(OrderStatus::Processed),
            "Awaiting Delivery" => Ok(OrderStatus::AwaitingDelivery),
            "Fulfilled" => Ok(OrderStatus::Fulfilled),
            "Cancelled" => Ok(OrderStatus::Cancelled),
            _ => Err("Invalid delivery status. Valid values are 'Pending', Payment Pending, Processed, 'Cancelled', 'Awaiting Delivery', 'Fulfilled'")

        }
    }
}
