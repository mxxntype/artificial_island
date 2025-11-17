use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::{Json, Router, routing};
use clap::Parser;
use epicentre_diagnostics::color_eyre::eyre::{self, Context};
use epicentre_diagnostics::tracing;
use tokio::sync::Mutex as AsyncMutex;
use tokio_util::sync::CancellationToken;

use crate::resource_monitor::{Metrics, ResourceMonitor};
use crate::{CLAP_STYLE, DEFAULT_API_ADDRESS};

#[derive(Parser, Debug)]
#[must_use]
#[command(version, author, about, styles(CLAP_STYLE))]
pub struct Options {
    /// Which address the [`axum`] server should bind to.
    #[arg(short('a'), long, default_value_t = DEFAULT_API_ADDRESS)]
    pub api_address: SocketAddr,

    /// The length of the unicode graph printed by this program, in characters.
    ///
    /// Note that the graph is printed using braille symbols, which means
    /// two distinct "measurements" can fit inside a single character.
    #[arg(short('l'), long, default_value_t = 5)]
    pub graph_length: u8,

    /// For how long, in seconds, to keep each *realtime* measurement in memory.
    ///
    /// Essentially defines the "lookback" period of the server, or the length
    /// of the produced graph in seconds, whatever makes more sense to you.
    #[arg(short('s'), long, default_value_t = 5)]
    pub span_seconds: u8,
}

#[tracing::instrument(name = "main")]
pub async fn run(options: &Options) -> Result<(), eyre::Error> {
    let measurement_capacity = options.graph_length * 2;
    let measurement_interval =
        Duration::from_secs_f64(f64::from(options.span_seconds) / f64::from(measurement_capacity));
    tracing::debug!(measurement_capacity, ?measurement_interval);

    let resource_monitor = ResourceMonitor::new(measurement_capacity.into());
    let resource_monitor = Arc::new(AsyncMutex::new(resource_monitor));

    let stop_signal = CancellationToken::new();

    tokio::select! {
        () = update_thread(
            Arc::clone(&resource_monitor),
            stop_signal.child_token(),
            measurement_interval
        ) => { /* never fails & returns nothing */ }

        axum_result = axum_thread(
            resource_monitor,
            stop_signal.child_token(),
            options.api_address
        ) => axum_result.wrap_err("The axum thread returned an error")?,

        signal = tokio::signal::ctrl_c() => {
            signal?;
            tracing::warn!("Received SIGINT, stopping the server");
        }
    }

    stop_signal.cancel();

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn update_thread(
    resource_monitor: Arc<AsyncMutex<ResourceMonitor>>,
    cancellation_token: CancellationToken,
    measurement_interval: Duration,
) {
    let update_loop = async move {
        loop {
            tokio::time::sleep(measurement_interval).await;
            let pre_refresh = Instant::now();
            resource_monitor.lock().await.refresh();
            tracing::trace!(refresh_duration = ?Instant::now().duration_since(pre_refresh));
        }
    };

    tokio::select! {
        () = update_loop => {}
        () = cancellation_token.cancelled() => {}
    }
}

#[tracing::instrument(skip_all)]
pub async fn axum_thread(
    resource_monitor: Arc<AsyncMutex<ResourceMonitor>>,
    cancellation_token: CancellationToken,
    socket_addr: SocketAddr,
) -> Result<(), std::io::Error> {
    let axum_future = async move {
        let state = AxumState { resource_monitor };
        let router = Router::new()
            .route("/metrics", routing::get(metrics_endpoint))
            .with_state(state);
        let listener = tokio::net::TcpListener::bind(&socket_addr)
            .await
            .inspect(|_| tracing::info!(?socket_addr, "Bound to socket"))?;
        axum::serve(listener, router).await
    };

    tokio::select! {
        axum_result = axum_future => axum_result,
        () = cancellation_token.cancelled() => Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct AxumState {
    resource_monitor: Arc<AsyncMutex<ResourceMonitor>>,
}

#[tracing::instrument(skip_all)]
#[axum::debug_handler]
pub async fn metrics_endpoint(State(state): State<AxumState>) -> Json<Metrics> {
    let metrics = state.resource_monitor.lock().await.build_metrics();
    Json(metrics)
}
