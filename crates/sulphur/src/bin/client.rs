use std::net::SocketAddr;

use clap::Parser;
use epicentre_diagnostics::{DiagnosticLayer, Report};
use sulphur::resource_monitor::{MeasurementType, Metrics};
use sulphur::{CLAP_STYLE, DEFAULT_API_ADDRESS, METRICS_ENDPOINT};

fn main() -> Result<(), Report> {
    DiagnosticLayer.setup()?;

    let options = ClientOptions::parse();
    let metrics = reqwest::blocking::get(options.metrics_http_uri())?.json::<Metrics>()?;
    let graph = sulphur::graph::render(&metrics, options.measurement_type)?;
    println!("{graph}");

    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, author, about, styles(CLAP_STYLE))]
pub struct ClientOptions {
    /// The address at which the server is configured to listen.
    #[arg(long, default_value_t = DEFAULT_API_ADDRESS)]
    pub api_address: SocketAddr,

    /// Graph of what metric type to render.
    #[arg(long("type"))]
    pub measurement_type: MeasurementType,
}

impl ClientOptions {
    #[must_use]
    pub fn metrics_http_uri(&self) -> String {
        let Self { api_address, .. } = self;
        format!("http://{api_address}{METRICS_ENDPOINT}")
    }
}
