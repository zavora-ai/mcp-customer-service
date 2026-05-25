mod client;

mod server;

use client::ApiClient;
use rmcp::{ServiceExt, transport::stdio};
use server::CsServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();
    let api = ApiClient::from_env()?;
    let service = CsServer { api }.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
