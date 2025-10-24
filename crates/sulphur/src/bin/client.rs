use std::net::SocketAddr;

use clap::Parser;
use epicentre_diagnostics::{DiagnosticLayer, Report};
use sulphur::Metrics;

fn main() -> Result<(), Report> {
    DiagnosticLayer.setup()?;

    let options = ClientOptions::parse();
    let uri = format!("http://{}/metrics", options.api_address);
    let metrics = reqwest::blocking::get(uri)?.json::<Metrics>()?;
    let graph = metrics.render_cpu_usage_graph();
    println!("{graph}");

    Ok(())
}

#[derive(Parser, Debug)]
struct ClientOptions {
    #[arg(long)]
    api_address: SocketAddr,
}
