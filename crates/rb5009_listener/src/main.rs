use axum::Router;
use axum::routing::get;
use epicentre_diagnostics::color_eyre::eyre::Report;
use epicentre_diagnostics::{DiagnosticLayer, tracing};
use std::net::Ipv4Addr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Report> {
    DiagnosticLayer.setup()?;

    let router = Router::new()
        .route("/wan/up", get(|| async { notify(WanStatus::Up).await }))
        .route("/wan/down", get(|| async { notify(WanStatus::Down).await }));
    let address = Ipv4Addr::UNSPECIFIED;
    let listener = TcpListener::bind((address, 2000)).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

pub enum WanStatus {
    Up,
    Down,
}

#[expect(clippy::unused_async)]
async fn notify(wan_status: WanStatus) {
    match wan_status {
        WanStatus::Up => tracing::info!("WAN is back online"),
        WanStatus::Down => tracing::warn!("WAN is fucked!"),
    }
}
