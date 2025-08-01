pub struct NumberToWords;

impl NumberToWords {
    const ONES: [&'static str; 20] = [
        "",
        "One",
        "Two",
        "Three",
        "Four",
        "Five",
        "Six",
        "Seven",
        "Eight",
        "Nine",
        "Ten",
        "Eleven",
        "Twelve",
        "Thirteen",
        "Fourteen",
        "Fifteen",
        "Sixteen",
        "Seventeen",
        "Eighteen",
        "Nineteen",
    ];

    const TENS: [&'static str; 10] = [
        "", "", "Twenty", "Thirty", "Forty", "Fifty", "Sixty", "Seventy", "Eighty", "Ninety",
    ];

    pub fn convert_to_words(amount: f64) -> String {
        if amount == 0.0 {
            return "Zero Rupees Only".to_string();
        }

        let rupees = amount.floor() as u64;
        let paisa = ((amount - amount.floor()) * 100.0).round() as u64;

        let mut result = String::new();

        if rupees > 0 {
            result.push_str(&Self::convert_rupees(rupees));
            result.push_str(" Rupees");
        }

        if paisa > 0 {
            if !result.is_empty() {
                result.push_str(" and ");
            }
            result.push_str(&Self::convert_paisa(paisa));
            result.push_str(" Paisa");
        }

        result.push_str(" Only");
        result
    }

    fn convert_rupees(mut num: u64) -> String {
        if num == 0 {
            return String::new();
        }

        let mut result = String::new();

        // Crores (10,000,000)
        if num >= 10_000_000 {
            let crores = num / 10_000_000;
            result.push_str(&Self::convert_hundreds(crores));
            result.push_str(" Crore");
            if crores > 1 {
                result.push('s');
            }
            num %= 10_000_000;
            if num > 0 {
                result.push(' ');
            }
        }

        // Lakhs (100,000)
        if num >= 100_000 {
            let lakhs = num / 100_000;
            result.push_str(&Self::convert_hundreds(lakhs));
            result.push_str(" Lakh");
            if lakhs > 1 {
                result.push('s');
            }
            num %= 100_000;
            if num > 0 {
                result.push(' ');
            }
        }

        // Thousands (1,000)
        if num >= 1_000 {
            let thousands = num / 1_000;
            result.push_str(&Self::convert_hundreds(thousands));
            result.push_str(" Thousand");
            num %= 1_000;
            if num > 0 {
                result.push(' ');
            }
        }

        // Hundreds
        if num > 0 {
            result.push_str(&Self::convert_hundreds(num));
        }

        result
    }

    fn convert_hundreds(mut num: u64) -> String {
        let mut result = String::new();

        if num >= 100 {
            let hundreds = num / 100;
            result.push_str(Self::ONES[hundreds as usize]);
            result.push_str(" Hundred");
            num %= 100;
            if num > 0 {
                result.push(' ');
            }
        }

        if num >= 20 {
            let tens = num / 10;
            result.push_str(Self::TENS[tens as usize]);
            num %= 10;
            if num > 0 {
                result.push(' ');
                result.push_str(Self::ONES[num as usize]);
            }
        } else if num > 0 {
            result.push_str(Self::ONES[num as usize]);
        }

        result
    }

    fn convert_paisa(num: u64) -> String {
        Self::convert_hundreds(num)
    }
}
