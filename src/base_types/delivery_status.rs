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
        match string_value.as_str(){
            "Pending" => Ok(DeliveryStatus::Pending),
            "On the way" => Ok(DeliveryStatus::OnTheWay),
            "Fulfilled" => Ok(DeliveryStatus::Fulfilled),
            "Cancelled" => Ok(DeliveryStatus::Cancelled),
            _ => Err("Invalid delivery status. Valid values are 'Peding', 'Cancelled', 'On the way', 'Fulfilled'")

        }
    }
}
