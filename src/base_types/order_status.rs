pub enum OrderStatus {
    PaymentPending,
    Pending,
    AwaitingDelivery,
    Fulfilled,
    Cancelled,
}

impl OrderStatus {
    pub fn value(&self) -> &str {
        match *self {
            OrderStatus::PaymentPending => "Payment Pending",
            OrderStatus::Pending => "Pending",
            OrderStatus::AwaitingDelivery => "Awaiting Delivery",
            OrderStatus::Fulfilled => "Fulfilled",
            OrderStatus::Cancelled => "Cancelled",
        }
    }

    pub fn from_str(string_value: &String) -> Result<Self, &str> {
        match string_value.as_str(){
            "Payment Pending" => Ok(OrderStatus::PaymentPending),
            "Pending" => Ok(OrderStatus::Pending),
            "Awaiting Delivery" => Ok(OrderStatus::AwaitingDelivery),
            "Fulfilled" => Ok(OrderStatus::Fulfilled),
            "Cancelled" => Ok(OrderStatus::Cancelled),
            _ => Err("Invalid delivery status. Valid values are 'Pending', Payment Pending, 'Cancelled', 'Awaiting Delivery', 'Fulfilled'")

        }
    }
}
