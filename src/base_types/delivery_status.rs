pub enum DeliveryStatus {
    Pending,
    OnTheWay,
    Fulfilled,
    Cancelled,
}

impl DeliveryStatus {
    pub fn value(&self) -> &str {
        match *self {
            DeliveryStatus::Pending => "Pending",
            DeliveryStatus::OnTheWay => "On the way",
            DeliveryStatus::Fulfilled => "Fulfilled",
            DeliveryStatus::Cancelled => "Cancelled",
        }
    }
}

