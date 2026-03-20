use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let api_key = std::env::var("ANNAS_ARCHIVE_API_KEY").ok();

    let client = match api_key {
        Some(key) => {
            tracing::info!("API key configured");
            annas_archive::Client::with_api_key(key)
        }
        None => {
            tracing::info!("no API key — search only");
            annas_archive::Client::new()
        }
    };

    let server = annas_archive::mcp::Server::new(Arc::new(client));

    tracing::info!("annas-archive MCP server starting on stdio");
    let service =
        rmcp::ServiceExt::serve(server, rmcp::transport::stdio()).await?;
    service.waiting().await?;

    Ok(())
}
