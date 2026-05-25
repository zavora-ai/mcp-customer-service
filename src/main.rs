mod domain;
mod server;
mod store;

use rmcp::{ServiceExt, transport::stdio};
use server::CsServer;
use store::Store;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();
    let service = CsServer { store: Store::seeded() }.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
