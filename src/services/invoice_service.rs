use anyhow::{Context, Result};
use diesel::prelude::Queryable;
use genpdf::{
    elements::{Break, LinearLayout, PaddedElement, Paragraph, TableLayout},
    style::Style,
    Alignment, Element,
};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

use crate::{config::CompanyConfiguration, utils::number_to_words::NumberToWords};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct InvoiceItem {
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub sku: String,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub invoice_number: String,
    pub print_count: i32,
    pub date: String,
    pub company_name: String,
    pub company_address: String,
    pub customer_name: String,
    pub customer_address: String,
    pub items: Vec<InvoiceItem>,
    pub subtotal: f64,
    pub discount: f64,
    pub tax_rate: f64,
    pub tax_amount: f64,
    pub total: f64,
}

#[derive(Debug)]
pub struct GeneratedInvoice {
    pub invoice: Invoice,
    pub pdf_bytes: Vec<u8>,
    pub file_size: u64,
}

// Main Invoice Service
pub struct InvoiceService {
    config: CompanyConfiguration,
}

impl InvoiceService {
    pub fn new(config: &CompanyConfiguration) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub async fn generate_invoice_pdf(
        &self,
        invoice_number: i32,
        print_count: i32,
        order_id: String,
        customer_name: String,
        customer_address: String,
        items: Vec<InvoiceItem>,
    ) -> Result<GeneratedInvoice> {
        let invoice = self.build_invoice_data(
            invoice_number,
            print_count,
            customer_name,
            customer_address,
            items,
        )?;

        let invoice_clone = invoice.clone();
        let order_id_clone = order_id.clone();

        let pdf_bytes = tokio::task::spawn_blocking(move || -> Result<Vec<u8>> {
            Self::generate_pdf_bytes(&invoice_clone, &order_id_clone)
        })
        .await
        .context("PDF generation task failed")?
        .context("PDF generation failed")?;

        let file_size = pdf_bytes.len() as u64;

        println!(
            "Invoice PDF generated successfully in memory: {} ({}KB)",
            invoice.invoice_number,
            file_size / 1024
        );

        Ok(GeneratedInvoice {
            invoice,
            pdf_bytes,
            file_size,
        })
    }

    fn build_invoice_data(
        &self,
        invoice_number: i32,
        print_count: i32,
        customer_name: String,
        customer_address: String,
        items: Vec<InvoiceItem>,
    ) -> Result<Invoice> {
        let subtotal = items.iter().map(|item| item.total).sum::<f64>();
        let tax_amount = subtotal * self.config.tax_rate;
        let total = subtotal + tax_amount;

        Ok(Invoice {
            invoice_number: format!("{:07}", invoice_number),
            print_count,
            date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            company_name: self.config.company_name.clone(),
            company_address: self.config.company_address.clone(),
            customer_name,
            customer_address,
            items,
            subtotal,
            discount: 0.0,
            tax_rate: self.config.tax_rate,
            tax_amount,
            total,
        })
    }

    fn generate_pdf_bytes(invoice: &Invoice, order_id: &String) -> Result<Vec<u8>> {
        use genpdf::{elements::*, fonts, Document};

        let font_family = fonts::from_files(
            "/usr/share/fonts/truetype/liberation",
            "LiberationSans",
            Some(genpdf::fonts::Builtin::Helvetica),
        )
        .context("Failed to load font family")?;

        let mut doc = Document::new(font_family);
        doc.set_title(&format!("Invoice {}", invoice.invoice_number));

        let mut decorator = genpdf::SimplePageDecorator::new();
        decorator.set_margins(10);
        doc.set_page_decorator(decorator);

        let mut layout = LinearLayout::vertical();

        layout.push(Self::create_header(invoice));
        layout.push(Break::new(2));
        layout.push(Self::create_invoice_details(invoice, order_id));
        layout.push(Break::new(2));
        layout.push(Self::create_customer_section(invoice));
        layout.push(Break::new(3));
        layout.push(Self::create_items_table(invoice));
        layout.push(Break::new(2));
        layout.push(Self::create_totals_section(invoice));
        layout.push(Break::new(1));
        layout.push(Self::create_amount_in_words(invoice));
        layout.push(Break::new(1));
        layout.push(Self::create_thank_you_section());

        doc.push(layout);

        doc.set_minimal_conformance(); //meta-data are removed

        // Generate PDF to a Vec<u8> in memory
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        doc.render(&mut cursor)
            .context("Failed to render PDF to memory")?;

        Ok(buffer)
    }

    fn create_header(invoice: &Invoice) -> LinearLayout {
        let mut header = LinearLayout::vertical();
        header.push(
            Paragraph::new(&invoice.company_name).styled(Style::new().bold().with_font_size(18)),
        );

        let address = invoice.company_address.replace("\\n", "\n");
        for line in address.lines() {
            header.push(Paragraph::new(line).styled(Style::new().with_font_size(10)));
        }
        header
    }

    fn create_invoice_details(invoice: &Invoice, order_id: &String) -> LinearLayout {
        let mut details = LinearLayout::vertical();
        details.push(Paragraph::new("INVOICE").styled(Style::new().bold().with_font_size(24)));
        details.push(Break::new(1));

        let invoice_title = match invoice.print_count {
            1 => format!("Invoice #: {}", invoice.invoice_number),
            2 => format!("Invoice #: {} - COPY", invoice.invoice_number),
            n if n > 2 => format!("Invoice #: {} - COPY ({})", invoice.invoice_number, n - 1),
            _ => format!("Invoice #: {}", invoice.invoice_number),
        };

        details.push(Paragraph::new(&invoice_title).styled(Style::new().bold()));
        details
            .push(Paragraph::new(&format!("Order Id #: {}", order_id)).styled(Style::new().bold()));
        details.push(Paragraph::new(&format!("Date: {}", invoice.date)).styled(Style::new()));

        details
    }

    fn create_customer_section(invoice: &Invoice) -> LinearLayout {
        let mut customer = LinearLayout::vertical();
        customer.push(Paragraph::new("Bill To:").styled(Style::new().bold().with_font_size(12)));
        customer.push(Paragraph::new(&invoice.customer_name).styled(Style::new().bold()));
        for line in invoice.customer_address.lines() {
            customer.push(Paragraph::new(line));
        }
        customer
    }

    fn create_items_table(invoice: &Invoice) -> TableLayout {
        let mut table = TableLayout::new(vec![1, 4, 2, 2, 2]);
        table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));

        table
            .row()
            .element(Self::header_cell("S.N."))
            .element(Self::header_cell("Description"))
            .element(Self::header_cell("Qty."))
            .element(Self::header_cell("Price (Rs)"))
            .element(Self::header_cell("Total (Rs)"))
            .push()
            .expect("Could not create table header");

        for (index, item) in invoice.items.iter().enumerate() {
            table
                .row()
                .element(Self::center_padded_paragraph(&format!("{}", index + 1)))
                .element(Self::left_padded_paragraph(&item.description))
                .element(Self::center_padded_paragraph(&format!(
                    "{:.2} {}",
                    item.quantity, item.sku
                )))
                .element(Self::right_padded_paragraph(&format!(
                    "{:.2}",
                    item.unit_price
                )))
                .element(Self::right_padded_paragraph(&format!("{:.2}", item.total)))
                .push()
                .unwrap();
        }

        table
    }

    fn create_totals_section(invoice: &Invoice) -> LinearLayout {
        let mut totals = LinearLayout::vertical();
        totals.push(Break::new(1));
        totals.push(
            Paragraph::new(&format!("Subtotal: Rs. {:.2}", invoice.subtotal))
                .aligned(Alignment::Right)
                .styled(Style::new().with_font_size(12)),
        );
        totals.push(
            Paragraph::new(&format!("Discount: Rs. {:.2}", invoice.discount))
                .aligned(Alignment::Right)
                .styled(Style::new().with_font_size(12)),
        );
        totals.push(
            Paragraph::new(&format!(
                "Tax ({:.1}%): Rs. {:.2}",
                invoice.tax_rate * 100.0,
                invoice.tax_amount
            ))
            .aligned(Alignment::Right)
            .styled(Style::new().with_font_size(12)),
        );
        totals.push(Break::new(0.5));
        totals.push(
            Paragraph::new(&format!("Total: Rs. {:.2}", invoice.total))
                .aligned(Alignment::Right)
                .styled(Style::new().bold().with_font_size(14)),
        );
        totals
    }

    fn create_amount_in_words(invoice: &Invoice) -> LinearLayout {
        let mut amount_section = LinearLayout::vertical();

        let amount_in_words = format!("Rupees {}", NumberToWords::convert_to_words(invoice.total));
        amount_section.push(
            Paragraph::new("Amount in Words:").styled(Style::new().bold().with_font_size(12)),
        );
        amount_section.push(
            Paragraph::new(&amount_in_words).styled(Style::new().italic().with_font_size(11)),
        );
        amount_section
    }

    fn create_thank_you_section() -> LinearLayout {
        let mut thank_you = LinearLayout::vertical();
        thank_you.push(
            Paragraph::new("THANK YOU for your purchase")
                .aligned(Alignment::Center)
                .styled(Style::new().bold().with_font_size(12)),
        );
        thank_you
    }

    fn header_cell(text: &str) -> PaddedElement<Paragraph> {
        Paragraph::default()
            .styled_string(text, Style::new().bold().with_font_size(12))
            .aligned(Alignment::Center)
            .padded(2)
    }

    fn left_padded_paragraph(text: &str) -> PaddedElement<Paragraph> {
        Paragraph::default()
            .styled_string(text, Style::new().with_font_size(11))
            .aligned(Alignment::Left)
            .padded(1)
    }

    fn right_padded_paragraph(text: &str) -> PaddedElement<Paragraph> {
        Paragraph::default()
            .styled_string(text, Style::new().with_font_size(11))
            .aligned(Alignment::Right)
            .padded(1)
    }

    fn center_padded_paragraph(text: &str) -> PaddedElement<Paragraph> {
        Paragraph::default()
            .styled_string(text, Style::new().with_font_size(11))
            .aligned(Alignment::Center)
            .padded(1)
    }
}

