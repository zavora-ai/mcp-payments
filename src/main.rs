mod client;
mod server;

use client::StripeClient;
use rmcp::{ServiceExt, transport::stdio};
use server::PaymentsServer;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse()?)).init();
    let key = std::env::var("STRIPE_SECRET_KEY").expect("STRIPE_SECRET_KEY required");
    let client = Arc::new(StripeClient::new(key));
    let server = PaymentsServer { client };
    tracing::info!("mcp-payments starting on stdio");
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
