use clap::Parser;
use epicentre_diagnostics::{DiagnosticLayer, Report};
use sulphur::server::Options;

#[tokio::main]
async fn main() -> Result<(), Report> {
    DiagnosticLayer.setup()?;

    let options = Options::parse();
    sulphur::server::run(&options).await?;

    Ok(())
}
