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
        match string_value.as_str() {
            "Cash" => Ok(PaymentMethod::Cash),
            "Esewa" => Ok(PaymentMethod::Esewa),
            "Khalti" => Ok(PaymentMethod::Khalti),
            "Bank Transfer" => Ok(PaymentMethod::BankTransfer),
            "Credit Card" => Ok(PaymentMethod::CreditCard),
            "Coupon" => Ok(PaymentMethod::Coupon),
            _ => Err("Invalid payment method. Valid values are 'Cash', 'Esewa', 'Khalti', 'Bank Transfer', 'Credit Card' and 'Coupon'")
        }
    }
}
