use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use super::{
    invoice_service::{Invoice, InvoiceItem},
    lettre_email_service::LettreEmailService,
};
use crate::config::EmailConfiguration;

#[async_trait]
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

    async fn send_html_email(
        &self,
        to_email: &str,
        subject: &str,
        html_body: &str,
        text_body: Option<&str>,
    ) -> Result<()>;

    async fn test_connection(&self) -> Result<()>;
}

pub struct EmailServiceFactory;

impl EmailServiceFactory {
    fn create_lettre_service(config: &EmailConfiguration) -> Result<Arc<dyn EmailService>> {
        let service = LettreEmailService::new(config)?;
        Ok(Arc::new(service))
    }

    pub fn create_gmail_service(
        gmail_config: &EmailConfiguration,
    ) -> Result<Arc<dyn EmailService>> {
        Self::create_lettre_service(gmail_config)
    }

    pub fn create_outlook_service(
        outlook_config: &EmailConfiguration,
    ) -> Result<Arc<dyn EmailService>> {
        Self::create_lettre_service(outlook_config)
    }
}
