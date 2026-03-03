//! MineClaw 服务器入口点

use std::net::SocketAddr;

use axum::serve;
use mineclaw::{create_router, create_provider, Config, AppState, SessionRepository};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> mineclaw::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let config = Config::load()?;
    info!("Configuration loaded successfully");

    let session_repo = SessionRepository::new();
    let llm_provider = create_provider(config.llm.clone());

    let app_state = AppState::new(session_repo, llm_provider);
    let app = create_router(app_state);

    let addr = SocketAddr::new(config.server.host.parse()?, config.server.port);
    let listener = TcpListener::bind(addr).await?;

    info!("MineClaw server listening on {}", addr);
    info!("Health check: http://{}/health", addr);

    serve(listener, app).await?;

    Ok(())
}