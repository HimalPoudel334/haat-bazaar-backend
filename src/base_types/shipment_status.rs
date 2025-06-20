pub enum ShipmentStatus {
    Pending,
    Processed,
    ReachedDeliveryFacility,
    AwaitingDelivery,
    OutForDelivery,
    Fulfilled,
    Cancelled,
}

impl ShipmentStatus {
    pub fn value(&self) -> &str {
        match *self {
            ShipmentStatus::Pending => "Pending",
            ShipmentStatus::Processed => "Processed",
            ShipmentStatus::ReachedDeliveryFacility => "Reached Delivery Facility",
            ShipmentStatus::AwaitingDelivery => "Awaiting Delivery",
            ShipmentStatus::OutForDelivery => "Out For Delivery",
            ShipmentStatus::Fulfilled => "Fulfilled",
            ShipmentStatus::Cancelled => "Cancelled",
        }
    }

    pub fn from_str(string_value: &String) -> Result<Self, &str> {
        let normalized = string_value.trim().to_lowercase();

        match normalized.as_str() {
            "pending" => Ok(ShipmentStatus::Pending),
            "processed" => Ok(ShipmentStatus::Processed),
            "reached delivery facility" => Ok(ShipmentStatus::ReachedDeliveryFacility),
            "awaiting delivery" => Ok(ShipmentStatus::AwaitingDelivery),
            "out for delivery" => Ok(ShipmentStatus::OutForDelivery),
            "fulfilled" => Ok(ShipmentStatus::Fulfilled),
            "cancelled" => Ok(ShipmentStatus::Cancelled),
            _ => Err("Invalid shipment status. Valid values are: 'Pending', 'Processed', 'Reached Delivery Facility', 'Awaiting Delivery', 'Out For Delivery', 'Fulfilled', 'Cancelled'"),
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            ShipmentStatus::Pending,
            ShipmentStatus::Processed,
            ShipmentStatus::ReachedDeliveryFacility,
            ShipmentStatus::AwaitingDelivery,
            ShipmentStatus::OutForDelivery,
            ShipmentStatus::Fulfilled,
            ShipmentStatus::Cancelled,
        ]
    }
}
