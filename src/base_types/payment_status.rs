#[derive(Eq, PartialEq)]
pub enum PaymentStatus {
    PaymentPending,
    Pending,
    Processed,
    Completed,
    Failed,
    Cancelled,
}

impl PaymentStatus {
    pub fn value(&self) -> &str {
        match *self {
            PaymentStatus::PaymentPending => "Payment Pending",
            PaymentStatus::Pending => "Pending",
            PaymentStatus::Processed => "Processed",
            PaymentStatus::Completed => "Completed",
            PaymentStatus::Failed => "Failed",
            PaymentStatus::Cancelled => "Cancelled",
        }
    }

    pub fn from_str(string_value: &String) -> Result<Self, &str> {
        let normalized = string_value.trim().to_lowercase();

        match normalized.as_str() {
            "payment pending" => Ok(PaymentStatus::PaymentPending),
            "pending" => Ok(PaymentStatus::Pending),
            "processed" => Ok(PaymentStatus::Processed),
            "Completed" => Ok(PaymentStatus::Completed),
            "fulfilled" => Ok(PaymentStatus::Failed),
            "cancelled" => Ok(PaymentStatus::Cancelled),
            _ => Err("Invalid payment status. Valid values are: 'Payment Pending', 'Pending', 'Processed', 'Completed', 'Failed', 'Cancelled'"),
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            PaymentStatus::PaymentPending,
            PaymentStatus::Pending,
            PaymentStatus::Processed,
            PaymentStatus::Completed,
            PaymentStatus::Failed,
            PaymentStatus::Cancelled,
        ]
    }
}
