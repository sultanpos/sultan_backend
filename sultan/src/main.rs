use dotenvy::dotenv;
use sultan::server::create_app;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let app = create_app().await?;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8721").await?;

    info!("Server listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
