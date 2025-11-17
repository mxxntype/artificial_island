use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::{Json, Router, routing};
use clap::Parser;
use epicentre_diagnostics::color_eyre::eyre::{self, Context};
use epicentre_diagnostics::tracing;
use tokio::sync::Mutex as AsyncMutex;
use tokio_util::sync::CancellationToken;

use crate::graph::GRAPH_DENSITY;
use crate::resource_monitor::{Metrics, ResourceMonitor, UpdateIntervals, realtime_update_thread};
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
    #[arg(short('s'), long, default_value_t = 5.0)]
    pub span_seconds: f64,
}

#[tracing::instrument(name = "main")]
pub async fn run(options: &Options) -> Result<(), eyre::Error> {
    let measurement_capacity = options.graph_length * GRAPH_DENSITY;
    let update_intervals = UpdateIntervals {
        realtime: Duration::from_secs_f64(options.span_seconds / f64::from(measurement_capacity)),
    };

    tracing::debug!(measurement_capacity, ?update_intervals);

    let resource_monitor = ResourceMonitor::new(measurement_capacity.into(), update_intervals);
    let resource_monitor = Arc::new(AsyncMutex::new(resource_monitor));
    let stop_signal = CancellationToken::new();

    tokio::select! {
        () = realtime_update_thread(
            Arc::clone(&resource_monitor),
            stop_signal.child_token(),
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
