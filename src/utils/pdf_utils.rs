use anyhow::{Context, Result};
use genpdf::{
    elements::{Break, LinearLayout, PaddedElement, Paragraph, TableLayout},
    style::Style,
    Alignment, Element,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub file_path: PathBuf,
    pub file_size: u64,
}

#[derive(Debug, Clone)]
pub struct InvoiceConfig {
    pub storage_dir: String,
    pub company_name: String,
    pub company_address: String,
    pub tax_rate: f64,
}

impl Default for InvoiceConfig {
    fn default() -> Self {
        Self {
            storage_dir: "/tmp/invoices".to_string(),
            company_name: "Your Company Name".to_string(),
            company_address:
                "123 Business St\nCity, State 12345\nPhone: (555) 123-4567\nEmail: info@company.com"
                    .to_string(),
            tax_rate: 0.085,
        }
    }
}

// Main Invoice Service
pub struct InvoiceService {
    config: InvoiceConfig,
}

impl InvoiceService {
    pub fn new(config: InvoiceConfig) -> Self {
        Self { config }
    }

    pub async fn generate_and_store_invoice(
        &self,
        order_id: String,
        customer_name: String,
        customer_address: String,
        items: Vec<InvoiceItem>,
    ) -> Result<GeneratedInvoice> {
        // Build invoice data
        let invoice = self.build_invoice_data(order_id, customer_name, customer_address, items)?;

        // Ensure storage directory exists
        self.ensure_storage_dir().await?;

        // Generate unique filename
        let filename = self.generate_filename(&invoice.invoice_number);
        let file_path = PathBuf::from(&self.config.storage_dir).join(&filename);

        // Generate PDF using spawn_blocking (blocking operation)
        let invoice_clone = invoice.clone();
        let file_path_clone = file_path.clone();

        tokio::task::spawn_blocking(move || -> Result<()> {
            Self::generate_pdf_blocking(&invoice_clone, &file_path_clone)
        })
        .await
        .context("PDF generation task failed")?
        .context("PDF generation failed")?;

        // Get file size for metadata
        let file_size = self.get_file_size(&file_path).await?;

        println!(
            "Invoice PDF generated successfully: {} ({}KB)",
            file_path.display(),
            file_size / 1024
        );

        Ok(GeneratedInvoice {
            invoice,
            file_path,
            file_size,
        })
    }

    /// Read invoice PDF as bytes (for email attachment)
    pub async fn read_invoice_bytes(&self, file_path: &Path) -> Result<Vec<u8>> {
        let bytes = fs::read(file_path)
            .await
            .context("Failed to read invoice PDF file")?;

        println!(
            "Read {} bytes from invoice file: {}",
            bytes.len(),
            file_path.display()
        );
        Ok(bytes)
    }

    /// Delete invoice file (cleanup after email sent)
    pub async fn cleanup_invoice_file(&self, file_path: &Path) -> Result<()> {
        if file_path.exists() {
            fs::remove_file(file_path)
                .await
                .context("Failed to delete invoice file")?;

            println!("Cleaned up invoice file: {}", file_path.display());
        }
        Ok(())
    }

    /// List all stored invoices (for admin purposes)
    pub async fn list_stored_invoices(&self) -> Result<Vec<PathBuf>> {
        let mut invoices = Vec::new();

        let mut dir = fs::read_dir(&self.config.storage_dir)
            .await
            .context("Failed to read invoice storage directory")?;

        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "pdf") {
                invoices.push(path);
            }
        }

        invoices.sort();
        Ok(invoices)
    }

    /// Get invoice file info without reading full content
    pub async fn get_invoice_info(&self, file_path: &Path) -> Result<InvoiceFileInfo> {
        let metadata = fs::metadata(file_path)
            .await
            .context("Failed to get file metadata")?;

        Ok(InvoiceFileInfo {
            path: file_path.to_path_buf(),
            size: metadata.len(),
            created: metadata.created().ok(),
            modified: metadata.modified().ok(),
        })
    }

    // Private helper methods

    fn build_invoice_data(
        &self,
        order_id: String,
        customer_name: String,
        customer_address: String,
        items: Vec<InvoiceItem>,
    ) -> Result<Invoice> {
        let subtotal = items.iter().map(|item| item.total).sum::<f64>();
        let tax_amount = subtotal * self.config.tax_rate;
        let total = subtotal + tax_amount;

        let invoice_number = format!(
            "INV-{}-{}",
            chrono::Utc::now().format("%Y%m%d"),
            &order_id.chars().take(8).collect::<String>().to_uppercase()
        );

        Ok(Invoice {
            invoice_number,
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

    async fn ensure_storage_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.config.storage_dir)
            .await
            .context("Failed to create invoice storage directory")?;
        Ok(())
    }

    fn generate_filename(&self, invoice_number: &str) -> String {
        format!(
            "{}_{}.pdf",
            invoice_number,
            Uuid::new_v4().to_string()[..8].to_lowercase()
        )
    }

    async fn get_file_size(&self, file_path: &Path) -> Result<u64> {
        let metadata = fs::metadata(file_path)
            .await
            .context("Failed to get file metadata")?;
        Ok(metadata.len())
    }

    // Blocking PDF generation (runs in spawn_blocking)
    fn generate_pdf_blocking(invoice: &Invoice, file_path: &Path) -> Result<()> {
        use genpdf::{elements::*, fonts, Document};

        let font_family = fonts::from_files("/usr/share/fonts/liberation/", "LiberationSans", None)
            .context("Failed to load font family")?;

        let mut doc = Document::new(font_family);
        doc.set_title(&format!("Invoice {}", invoice.invoice_number));

        let mut decorator = genpdf::SimplePageDecorator::new();
        decorator.set_margins(10);
        doc.set_page_decorator(decorator);

        let mut layout = LinearLayout::vertical();

        // Build PDF content
        layout.push(Self::create_header(invoice));
        layout.push(Break::new(2));
        layout.push(Self::create_invoice_details(invoice));
        layout.push(Break::new(2));
        layout.push(Self::create_customer_section(invoice));
        layout.push(Break::new(3));
        layout.push(Self::create_items_table(invoice));
        layout.push(Break::new(2));
        layout.push(Self::create_totals_section(invoice));
        layout.push(Break::new(1));
        layout.push(Self::create_amount_in_words(invoice));

        doc.push(layout);
        doc.render_to_file(file_path)
            .context("Failed to render PDF to file")?;

        Ok(())
    }

    fn create_header(invoice: &Invoice) -> LinearLayout {
        let mut header = LinearLayout::vertical();
        header.push(
            Paragraph::new(&invoice.company_name).styled(Style::new().bold().with_font_size(18)),
        );
        for line in invoice.company_address.lines() {
            header.push(Paragraph::new(line).styled(Style::new().with_font_size(10)));
        }
        header
    }

    fn create_invoice_details(invoice: &Invoice) -> LinearLayout {
        let mut details = LinearLayout::vertical();
        details.push(Paragraph::new("INVOICE").styled(Style::new().bold().with_font_size(24)));
        details.push(Break::new(1));
        details.push(
            Paragraph::new(&format!("Invoice #: {}", invoice.invoice_number))
                .styled(Style::new().bold()),
        );
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
                .styled(Style::new().bold().with_font_size(14)),
        );
        totals
    }

    fn create_amount_in_words(invoice: &Invoice) -> LinearLayout {
        let mut amount_section = LinearLayout::vertical();
        // You'll need to implement NumberToWords or use a crate
        let amount_in_words = format!("Rupees {} only", invoice.total as u64); // Simplified
        amount_section.push(
            Paragraph::new("Amount in Words:").styled(Style::new().bold().with_font_size(12)),
        );
        amount_section.push(
            Paragraph::new(&amount_in_words).styled(Style::new().italic().with_font_size(11)),
        );
        amount_section
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

// Helper structs
#[derive(Debug)]
pub struct InvoiceFileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub created: Option<std::time::SystemTime>,
    pub modified: Option<std::time::SystemTime>,
}
