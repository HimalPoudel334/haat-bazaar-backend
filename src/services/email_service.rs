use anyhow::{Context, Result};
use lettre::{
    message::{header, Attachment, Body, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use tokio::time::{timeout, Duration};

use crate::utils::pdf_utils::{Invoice, InvoiceItem};

// Email service trait
#[async_trait::async_trait]
pub trait EmailService: Send + Sync {
    async fn send_invoice_email(
        &self,
        to_email: &str,
        customer_name: &str,
        invoice: &Invoice,
        pdf_bytes: &[u8],
    ) -> Result<()>;

    async fn send_order_confirmation_email(
        &self,
        to_email: &str,
        customer_name: &str,
        order_id: &str,
        order_details: &str,
    ) -> Result<()>;

    async fn send_simple_email(&self, to_email: &str, subject: &str, body: &str) -> Result<()>;
}

// Email configuration
#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
    pub timeout_seconds: u64,
    pub use_tls: bool,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp_server: "smtp.gmail.com".to_string(),
            smtp_port: 587,
            username: "himalpou101@gmail.com".to_string(),
            password: "your-app-password".to_string(),
            from_email: "himalpou101@gmail.com".to_string(),
            from_name: "Haatbazar".to_string(),
            timeout_seconds: 30,
            use_tls: true,
        }
    }
}

// Lettre-based Email Service Implementation
pub struct LettreEmailService {
    config: EmailConfig,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl LettreEmailService {
    pub fn new(config: EmailConfig) -> Result<Self> {
        let mailer = Self::create_mailer(&config)?;
        Ok(Self { config, mailer })
    }

    fn create_mailer(config: &EmailConfig) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
        let creds = Credentials::new(config.username.clone(), config.password.clone());

        let mailer_builder =
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
                .context("Failed to create STARTTLS SMTP relay")?
                .credentials(creds)
                .timeout(Some(std::time::Duration::from_secs(config.timeout_seconds)));

        let mailer = if config.use_tls {
            mailer_builder.build()
        } else {
            mailer_builder.build()
        };

        Ok(mailer)
    }
}

#[async_trait::async_trait]
impl EmailService for LettreEmailService {
    async fn send_invoice_email(
        &self,
        to_email: &str,
        customer_name: &str,
        invoice: &Invoice,
        pdf_bytes: &[u8],
    ) -> Result<()> {
        let subject = format!(
            "Invoice {} - {}",
            invoice.invoice_number, self.config.from_name
        );
        let body = self.build_invoice_email_body(customer_name, invoice);
        let attachment_filename = format!("invoice_{}.pdf", invoice.invoice_number);

        self.send_email_with_pdf_attachment(
            to_email,
            &subject,
            &body,
            pdf_bytes,
            &attachment_filename,
        )
        .await
        .context("Failed to send invoice email")?;

        println!(
            "Invoice email sent successfully to {} for invoice {}",
            to_email, invoice.invoice_number
        );

        Ok(())
    }

    async fn send_order_confirmation_email(
        &self,
        to_email: &str,
        customer_name: &str,
        order_id: &str,
        order_details: &str,
    ) -> Result<()> {
        let subject = format!(
            "Order Confirmation #{} - {}",
            order_id, self.config.from_name
        );
        let body = self.build_order_confirmation_body(customer_name, order_id, order_details);

        self.send_simple_text_email(to_email, &subject, &body)
            .await
            .context("Failed to send order confirmation email")?;

        println!(
            "Order confirmation email sent to {} for order {}",
            to_email, order_id
        );
        Ok(())
    }

    async fn send_simple_email(&self, to_email: &str, subject: &str, body: &str) -> Result<()> {
        self.send_simple_text_email(to_email, subject, body)
            .await
            .context("Failed to send simple email")?;

        println!("Simple email sent to {}: {}", to_email, subject);
        Ok(())
    }
}

impl LettreEmailService {
    // Send email with PDF attachment
    async fn send_email_with_pdf_attachment(
        &self,
        to_email: &str,
        subject: &str,
        body: &str,
        pdf_bytes: &[u8],
        filename: &str,
    ) -> Result<()> {
        let from_address = format!("{} <{}>", self.config.from_name, self.config.from_email);

        // Create text part
        let text_part = SinglePart::builder()
            .header(header::ContentType::TEXT_PLAIN)
            .body(String::from(body));

        // Create PDF attachment
        let attachment = Attachment::new(String::from(filename))
            .body(pdf_bytes.to_vec(), "application/pdf".parse().unwrap());

        // Build multipart email
        let email = Message::builder()
            .from(from_address.parse().context("Invalid from address")?)
            .to(to_email.parse().context("Invalid to address")?)
            .subject(subject)
            .multipart(
                MultiPart::mixed()
                    .singlepart(text_part)
                    .singlepart(attachment),
            )
            .context("Failed to build email message")?;

        // Send with timeout
        self.send_message_with_timeout(email).await?;

        Ok(())
    }

    // Send simple text email
    async fn send_simple_text_email(
        &self,
        to_email: &str,
        subject: &str,
        body: &str,
    ) -> Result<()> {
        let from_address = format!("{} <{}>", self.config.from_name, self.config.from_email);

        let email = Message::builder()
            .from(from_address.parse().context("Invalid from address")?)
            .to(to_email.parse().context("Invalid to address")?)
            .subject(subject)
            .body(String::from(body))
            .context("Failed to build email message")?;

        self.send_message_with_timeout(email).await?;

        Ok(())
    }

    pub async fn send_html_email(
        &self,
        to_email: &str,
        subject: &str,
        html_body: &str,
        text_body: Option<&str>,
    ) -> Result<()> {
        let from_address = format!("{} <{}>", self.config.from_name, self.config.from_email);

        let email = if let Some(text) = text_body {
            // Multipart alternative (HTML + text fallback)
            let html_part = SinglePart::builder()
                .header(header::ContentType::TEXT_HTML)
                .body(String::from(html_body));

            let text_part = SinglePart::builder()
                .header(header::ContentType::TEXT_PLAIN)
                .body(String::from(text));

            Message::builder()
                .from(from_address.parse().context("Invalid from address")?)
                .to(to_email.parse().context("Invalid to address")?)
                .subject(subject)
                .multipart(
                    MultiPart::alternative()
                        .singlepart(text_part)
                        .singlepart(html_part),
                )
                .context("Failed to build HTML email message")?
        } else {
            // HTML only
            Message::builder()
                .from(from_address.parse().context("Invalid from address")?)
                .to(to_email.parse().context("Invalid to address")?)
                .subject(subject)
                .header(header::ContentType::TEXT_HTML)
                .body(String::from(html_body))
                .context("Failed to build HTML email message")?
        };

        self.send_message_with_timeout(email).await?;

        println!("HTML email sent to {}: {}", to_email, subject);
        Ok(())
    }

    // Core sending method with timeout
    async fn send_message_with_timeout(&self, email: Message) -> Result<()> {
        timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.mailer.send(email),
        )
        .await
        .context("Email sending timed out")?
        .context("SMTP sending failed")?;

        Ok(())
    }

    // Email template builders
    fn build_invoice_email_body(&self, customer_name: &str, invoice: &Invoice) -> String {
        format!(
            r#"Dear {},

Thank you for your order! Please find your invoice attached.

Invoice Details:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Invoice Number: {}
Date: {}
Total Amount: Rs. {:.2}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Items:
{}

If you have any questions about this invoice, please don't hesitate to contact us.

Best regards,
{}

---
This is an automated message. Please do not reply to this email.
"#,
            customer_name,
            invoice.invoice_number,
            invoice.date,
            invoice.total,
            self.format_invoice_items(&invoice.items),
            self.config.from_name
        )
    }

    fn build_order_confirmation_body(
        &self,
        customer_name: &str,
        order_id: &str,
        order_details: &str,
    ) -> String {
        format!(
            r#"Dear {},

Thank you for your order! We have received your order and it is being processed.

Order Details:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Order ID: {}
Order Summary: {}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

You will receive an invoice shortly via email.

We will notify you once your order is ready for delivery.

Best regards,
{}

---
This is an automated message. Please do not reply to this email.
"#,
            customer_name, order_id, order_details, self.config.from_name
        )
    }

    fn format_invoice_items(&self, items: &[InvoiceItem]) -> String {
        items
            .iter()
            .map(|item| {
                format!(
                    "• {} x{:.1} {} @ Rs.{:.2} = Rs.{:.2}",
                    item.description, item.quantity, item.sku, item.unit_price, item.total
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    // Test connection method
    pub async fn test_connection(&self) -> Result<()> {
        timeout(Duration::from_secs(10), self.mailer.test_connection())
            .await
            .context("Connection test timed out")?
            .context("SMTP connection test failed")?;

        println!("SMTP connection test successful");
        Ok(())
    }
}

// Factory for creating email service
pub struct EmailServiceFactory;

impl EmailServiceFactory {
    pub fn create_lettre_service(config: EmailConfig) -> Result<Box<dyn EmailService>> {
        let service = LettreEmailService::new(config)?;
        Ok(Box::new(service))
    }

    // Gmail-specific factory method
    pub fn create_gmail_service(
        email: String,
        app_password: String,
        from_name: String,
    ) -> Result<Box<dyn EmailService>> {
        let config = EmailConfig {
            smtp_server: "smtp.gmail.com".to_string(),
            smtp_port: 587,
            username: email.clone(),
            password: app_password,
            from_email: email,
            from_name,
            timeout_seconds: 30,
            use_tls: true,
        };

        Self::create_lettre_service(config)
    }

    // Outlook-specific factory method
    pub fn create_outlook_service(
        email: String,
        password: String,
        from_name: String,
    ) -> Result<Box<dyn EmailService>> {
        let config = EmailConfig {
            smtp_server: "smtp-mail.outlook.com".to_string(),
            smtp_port: 587,
            username: email.clone(),
            password,
            from_email: email,
            from_name,
            timeout_seconds: 30,
            use_tls: true,
        };

        Self::create_lettre_service(config)
    }
}
