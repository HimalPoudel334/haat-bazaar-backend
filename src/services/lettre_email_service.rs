use anyhow::{Context, Result};
use async_trait::async_trait;
use lettre::{
    message::{header, Attachment, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use tokio::time::{timeout, Duration};

use super::email_service::{EmailService, EmailServiceFactory};
use crate::config::EmailConfiguration;
use crate::services::invoice_service::{Invoice, InvoiceItem};

pub struct LettreEmailService {
    config: EmailConfiguration,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl LettreEmailService {
    pub fn new(config: &EmailConfiguration) -> Result<Self> {
        let mailer = Self::create_mailer(config)?;
        Ok(Self {
            config: config.to_owned(),
            mailer,
        })
    }

    fn create_mailer(config: &EmailConfiguration) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
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

    fn build_invoice_email_body(&self, customer_name: &str, invoice: &Invoice) -> String {
        format!(
            r#"Dear {},

Thank you for your order! Please find your invoice attached.

Invoice Details:
━━━━━━━━━━━━━━━━━━━━━━━━
Invoice Number: {}
Date: {}
Total Amount: Rs. {:.2}
━━━━━━━━━━━━━━━━━━━━━━━━

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
━━━━━━━━━━━━━━━━━━
Order ID: {}
Order Summary: {}
━━━━━━━━━━━━━━━━━━

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

    async fn send_email_with_pdf_attachment(
        &self,
        to_email: &str,
        subject: &str,
        body: &str,
        pdf_bytes: &[u8],
        filename: &str,
    ) -> Result<()> {
        let from_address = format!("{} <{}>", self.config.from_name, self.config.from_email);

        let text_part = SinglePart::builder()
            .header(header::ContentType::TEXT_PLAIN)
            .body(String::from(body));

        let attachment = Attachment::new(String::from(filename))
            .body(pdf_bytes.to_vec(), "application/pdf".parse().unwrap());

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

        self.send_message_with_timeout(email).await?;

        Ok(())
    }

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
}

#[async_trait]
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

    async fn send_html_email(
        &self,
        to_email: &str,
        subject: &str,
        html_body: &str,
        text_body: Option<&str>,
    ) -> Result<()> {
        let from_address = format!("{} <{}>", self.config.from_name, self.config.from_email);

        let email = if let Some(text) = text_body {
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

    async fn test_connection(&self) -> Result<()> {
        timeout(Duration::from_secs(10), self.mailer.test_connection())
            .await
            .context("Connection test timed out")?
            .context("SMTP connection test failed")?;

        println!("SMTP connection test successful");
        Ok(())
    }
}
