mod backend;
mod client;
mod freshdesk;
mod intercom;
mod server;
mod zendesk;

use backend::Backend;
use rmcp::{ServiceExt, transport::stdio};
use server::CsServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();
    let backend = Backend::from_env()?;
    let service = CsServer { backend }.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
