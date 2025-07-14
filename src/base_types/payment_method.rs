#[derive(Eq, PartialEq)]
pub enum PaymentMethod {
    Cash,
    Esewa,
    Khalti,
    BankTransfer,
    Coupon,
    CreditCard,
}

impl PaymentMethod {
    pub fn value(&self) -> &str {
        match *self {
            PaymentMethod::Cash => "Cash",
            PaymentMethod::Esewa => "Esewa",
            PaymentMethod::Khalti => "Khalti",
            PaymentMethod::BankTransfer => "Bank Transfer",
            PaymentMethod::CreditCard => "Credit Card",
            PaymentMethod::Coupon => "Coupon",
        }
    }

    pub fn from_str(string_value: &String) -> Result<Self, &str> {
        let normalized = string_value.trim().to_lowercase();

        match normalized.as_str() {
            "cash" => Ok(PaymentMethod::Cash),
            "esewa" => Ok(PaymentMethod::Esewa),
            "khalti" => Ok(PaymentMethod::Khalti),
            "bank transfer" => Ok(PaymentMethod::BankTransfer),
            "credit card" => Ok(PaymentMethod::CreditCard),
            "coupon" => Ok(PaymentMethod::Coupon),
            _ => Err("Invalid payment method. Valid values are 'Cash', 'Esewa', 'Khalti', 'Bank Transfer', 'Credit Card' and 'Coupon'"),
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            PaymentMethod::Cash,
            PaymentMethod::Esewa,
            PaymentMethod::Khalti,
            PaymentMethod::BankTransfer,
            PaymentMethod::Coupon,
            PaymentMethod::CreditCard,
        ]
    }
}
