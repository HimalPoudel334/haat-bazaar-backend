pub enum PaymentMethod {
    Cash,
    Esewa,
    BankTransfer,
    Coupon,
    CreditCard,
}

impl PaymentMethod {
    pub fn value(&self) -> &str {
        match *self {
            PaymentMethod::Cash => "Cash",
            PaymentMethod::Esewa => "Esewa",
            PaymentMethod::BankTransfer => "Bank Transfer",
            PaymentMethod::CreditCard => "CreditCard",
            PaymentMethod::Coupon => "Coupon",
        }
    }
}
