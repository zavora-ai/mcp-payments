mod domain;
mod store;
mod server;

use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let store = store::PaymentStore::new();
    let server = server::PaymentServer { store };
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
