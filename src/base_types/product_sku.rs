pub enum ProductSKU {
    Kg,
    Litre,
    Bundle,
    Dozen,
    Piece,
    Quintal,
}

impl ProductSKU {
    pub fn value(&self) -> &str {
        match *self {
            ProductSKU::Kg => "Kg",
            ProductSKU::Litre => "Litre",
            ProductSKU::Bundle => "Bundle",
            ProductSKU::Dozen => "Dozen",
            ProductSKU::Piece => "Piece",
            ProductSKU::Quintal => "Qunital",
        }
    }

    pub fn from_str(string_value: &String) -> Result<Self, &str> {
        let normalized = string_value.trim().to_lowercase();
        match normalized.as_str(){
            "kg" => Ok(ProductSKU::Kg),
            "litre" => Ok(ProductSKU::Litre),
            "bundle" => Ok(ProductSKU::Bundle),
            "dozen" => Ok(ProductSKU::Dozen),
            "piece" => Ok(ProductSKU::Piece),
            "quintal" => Ok(ProductSKU::Quintal),
            _ => Err("Invalid Stock keeping unit. Valid values are 'Kg', 'Bundle', 'Litre', 'Dozen', 'Piece', 'Qunital'")

        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            ProductSKU::Kg,
            ProductSKU::Litre,
            ProductSKU::Bundle,
            ProductSKU::Dozen,
            ProductSKU::Piece,
            ProductSKU::Quintal,
        ]
    }
}
