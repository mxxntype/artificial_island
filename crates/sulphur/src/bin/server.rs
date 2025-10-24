use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::{Json, Router, routing};
use clap::Parser;
use epicentre_diagnostics::{DiagnosticLayer, Report, tracing};
use sulphur::{Metrics, ResourceMonitor, ServerOptions};
use tokio::sync::Mutex as AsyncMutex;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<(), Report> {
    DiagnosticLayer.setup()?;

    let options = ServerOptions::parse();
    tracing::debug!(?options);

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
        ) => {}

        _ = axum_thread(
            resource_monitor,
            stop_signal.child_token(),
            options.listen_addr
        ) => {}

        signal = tokio::signal::ctrl_c() => match signal {
            Ok(()) => tracing::warn!("Received SIGINT, stopping the server"),
            Err(error) => tracing::error!(?error, "Failed to await SIGINT"),
        }
    }

    stop_signal.cancel();

    Ok(())
}

#[tracing::instrument(skip_all)]
async fn update_thread(
    resource_monitor: Arc<AsyncMutex<ResourceMonitor>>,
    cancellation_token: CancellationToken,
    measurement_interval: Duration,
) {
    let update_loop = async move {
        loop {
            tokio::time::sleep(measurement_interval).await;
            let pre_refresh = Instant::now();
            resource_monitor.lock().await.refresh();
            tracing::debug!(refresh_duration = ?Instant::now().duration_since(pre_refresh));
        }
    };

    tokio::select! {
        () = update_loop => {}
        () = cancellation_token.cancelled() => {}
    }
}

#[tracing::instrument(skip_all)]
async fn axum_thread(
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
            .inspect(|_| tracing::debug!(?socket_addr, "Bound to socket"))?;
        axum::serve(listener, router).await
    };

    tokio::select! {
        axum_result = axum_future => axum_result,
        () = cancellation_token.cancelled() => Ok(())
    }
}

#[derive(Clone, Debug)]
struct AxumState {
    resource_monitor: Arc<AsyncMutex<ResourceMonitor>>,
}

#[tracing::instrument(skip_all)]
#[axum::debug_handler]
async fn metrics_endpoint(State(state): State<AxumState>) -> Json<Metrics> {
    let metrics = state.resource_monitor.lock().await.build_metrics();
    Json(metrics)
}
