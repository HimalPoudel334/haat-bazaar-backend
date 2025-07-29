use genpdf::elements::{Break, LinearLayout, PaddedElement, Paragraph, TableLayout};
use genpdf::fonts::{self};
use genpdf::{style::Style, Alignment};
use genpdf::{Document, Element};

use super::number_to_words::NumberToWords;

pub struct InvoiceItem {
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub sku: String,
    pub total: f64,
}

pub struct Invoice {
    pub invoice_number: String,
    pub date: String,
    pub company_name: String,
    pub company_address: String,
    pub customer_name: String,
    pub customer_address: String,
    pub items: Vec<InvoiceItem>,
    pub subtotal: f64,
    pub tax_rate: f64,
    pub tax_amount: f64,
    pub total: f64,
}

impl Invoice {
    pub fn generate_pdf(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let font_family =
            fonts::from_files("/usr/share/fonts/liberation/", "LiberationSans", None)?;
        let mut doc = Document::new(font_family);
        doc.set_title(&format!("Invoice {}", self.invoice_number));

        let mut decorator = genpdf::SimplePageDecorator::new();
        decorator.set_margins(10);
        doc.set_page_decorator(decorator);

        let mut layout = LinearLayout::vertical();

        layout.push(self.create_header());
        layout.push(Break::new(2));

        layout.push(self.create_invoice_details());
        layout.push(Break::new(2));

        layout.push(self.create_customer_section());
        layout.push(Break::new(3));

        layout.push(self.create_items_table());
        layout.push(Break::new(2));

        layout.push(self.create_totals_section());
        layout.push(Break::new(1));

        layout.push(self.create_amount_in_words());

        doc.push(layout);
        doc.render_to_file(filename)?;

        Ok(())
    }

    fn create_header(&self) -> LinearLayout {
        let mut header = LinearLayout::vertical();

        header.push(
            Paragraph::new(&self.company_name).styled(Style::new().bold().with_font_size(18)),
        );

        for line in self.company_address.lines() {
            header.push(Paragraph::new(line).styled(Style::new().with_font_size(10)));
        }

        header
    }

    fn create_invoice_details(&self) -> LinearLayout {
        let mut details = LinearLayout::vertical();

        details.push(Paragraph::new("INVOICE").styled(Style::new().bold().with_font_size(24)));

        details.push(Break::new(1));

        details.push(
            Paragraph::new(&format!("Invoice #: {}", self.invoice_number))
                .styled(Style::new().bold()),
        );
        details.push(Paragraph::new(&format!("Date: {}", self.date)).styled(Style::new()));

        details
    }

    fn create_customer_section(&self) -> LinearLayout {
        let mut customer = LinearLayout::vertical();

        customer.push(Paragraph::new("Bill To:").styled(Style::new().bold().with_font_size(12)));

        customer.push(Paragraph::new(&self.customer_name).styled(Style::new().bold()));

        for line in self.customer_address.lines() {
            customer.push(Paragraph::new(line));
        }

        customer
    }

    fn create_items_table(&self) -> TableLayout {
        let mut table = TableLayout::new(vec![1, 4, 2, 2, 2]);
        table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));

        table
            .row()
            .element(self.header_cell("S.N."))
            .element(self.header_cell("Description"))
            .element(self.header_cell("Qty."))
            .element(self.header_cell("Price (Rs)"))
            .element(self.header_cell("Total (Rs)"))
            .push()
            .expect("Could not create table header");

        for (index, item) in self.items.iter().enumerate() {
            table
                .row()
                .element(self.center_padded_paragraph(&format!("{}", index + 1)))
                .element(self.left_padded_paragraph(&item.description))
                .element(
                    self.center_padded_paragraph(&format!("{:.2} {}", item.quantity, item.sku)),
                )
                .element(self.right_padded_paragraph(&format!("{:.2}", item.unit_price)))
                .element(self.right_padded_paragraph(&format!("{:.2}", item.total)))
                .push()
                .unwrap();
        }

        table
    }

    fn create_totals_section(&self) -> LinearLayout {
        let mut totals = LinearLayout::vertical();

        totals.push(Break::new(1));

        totals.push(
            Paragraph::new(&format!("Subtotal: Rs. {:.2}", self.subtotal))
                .styled(Style::new().with_font_size(12)),
        );
        totals.push(
            Paragraph::new(&format!(
                "Tax ({:.1}%): Rs. {:.2}",
                self.tax_rate * 100.0,
                self.tax_amount
            ))
            .styled(Style::new().with_font_size(12)),
        );
        totals.push(Break::new(0.5));
        totals.push(
            Paragraph::new(&format!("Total: Rs. {:.2}", self.total))
                .styled(Style::new().bold().with_font_size(14)),
        );

        totals
    }

    fn create_amount_in_words(&self) -> LinearLayout {
        let mut amount_section = LinearLayout::vertical();

        let amount_in_words = NumberToWords::convert_to_words(self.total);

        amount_section.push(
            Paragraph::new("Amount in Words:").styled(Style::new().bold().with_font_size(12)),
        );

        amount_section.push(
            Paragraph::new(&amount_in_words).styled(Style::new().italic().with_font_size(11)),
        );

        amount_section
    }

    fn header_cell(&self, text: &str) -> PaddedElement<Paragraph> {
        Paragraph::default()
            .styled_string(text, Style::new().bold().with_font_size(12))
            .aligned(Alignment::Center)
            .padded(2)
    }

    fn left_padded_paragraph(&self, text: &str) -> PaddedElement<Paragraph> {
        Paragraph::default()
            .styled_string(text, Style::new().with_font_size(11))
            .aligned(Alignment::Left)
            .padded(1)
    }

    fn right_padded_paragraph(&self, text: &str) -> PaddedElement<Paragraph> {
        Paragraph::default()
            .styled_string(text, Style::new().with_font_size(11))
            .aligned(Alignment::Right)
            .padded(1)
    }

    fn center_padded_paragraph(&self, text: &str) -> PaddedElement<Paragraph> {
        Paragraph::default()
            .styled_string(text, Style::new().with_font_size(11))
            .aligned(Alignment::Center)
            .padded(1)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let items = vec![
        InvoiceItem {
            description: "Web Development Services".to_string(),
            quantity: 40.0,
            unit_price: 7500.0,
            sku: "Pcs".to_string(),
            total: 300000.0,
        },
        InvoiceItem {
            description: "Domain Registration".to_string(),
            quantity: 10.0,
            unit_price: 1500.0,
            sku: "litres".to_string(),
            total: 15000.0,
        },
        InvoiceItem {
            description: "Hosting (1 year)".to_string(),
            quantity: 12.0,
            unit_price: 1200.0,
            sku: "Kg".to_string(),
            total: 14400.0,
        },
    ];

    let invoice = Invoice {
        invoice_number: "INV-2024-001".to_string(),
        date: "2024-01-15".to_string(),
        company_name: "Your Company Name".to_string(),
        company_address:
            "123 Business St\nCity, State 12345\nPhone: (555) 123-4567\nEmail: info@company.com"
                .to_string(),
        customer_name: "Customer Name".to_string(),
        customer_address: "456 Customer Ave\nCustomer City, State 67890".to_string(),
        items,
        subtotal: 3135.0,
        tax_rate: 0.085,
        tax_amount: 266.48,
        total: 3401.48,
    };

    invoice.generate_pdf("/mnt/d/Himal/invoice.pdf")?;
    println!("Invoice PDF generated successfully!");

    Ok(())
}
