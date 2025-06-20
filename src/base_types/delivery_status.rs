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

    pub fn from_str(string_value: &String) -> Result<Self, &str> {
        let normalized = string_value.trim().to_lowercase();
        match normalized.as_str(){
            "pending" => Ok(DeliveryStatus::Pending),
            "on the way" => Ok(DeliveryStatus::OnTheWay),
            "fulfilled" => Ok(DeliveryStatus::Fulfilled),
            "cancelled" => Ok(DeliveryStatus::Cancelled),
            _ => Err("Invalid delivery status. Valid values are 'Peding', 'Cancelled', 'On the way', 'Fulfilled'")

        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            DeliveryStatus::Pending,
            DeliveryStatus::OnTheWay,
            DeliveryStatus::Fulfilled,
            DeliveryStatus::Cancelled,
        ]
    }
}
