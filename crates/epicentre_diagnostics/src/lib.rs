pub use color_eyre::eyre::Report;
pub use {color_eyre, tracing, tracing_error, tracing_subscriber};

/// Over-engineered diagnostics layer.
///
/// This "layer" is a [`tracing_subscriber`] coupled with a [`color_eyre`] panic
/// hook, together with a [`tracing_error`] error layer and some opinionated
/// defaults for the format layer. In conjunction, these provide an ergonomic
/// way of both logging errors and presenting them to the user (if applicable).
#[derive(Debug)]
#[must_use]
pub struct DiagnosticLayer;

impl DiagnosticLayer {
    /// Setup an application-side diagnostic layer.
    ///
    /// # Errors
    ///
    /// This function will return an error if the [`color_eyre::install`] call
    /// fails, or if the [`tracing_subscriber`] can't be installed.
    #[tracing::instrument(skip_all)]
    pub fn setup(&self) -> Result<(), color_eyre::eyre::Error> {
        use tracing_error::ErrorLayer;
        use tracing_subscriber::prelude::*;
        use tracing_subscriber::{fmt, EnvFilter};

        color_eyre::install()?;

        let filter_layer = EnvFilter::try_from_default_env().unwrap_or_default();
        let format_layer = fmt::layer().pretty().with_writer(std::io::stderr);
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(format_layer)
            .with(ErrorLayer::default())
            .try_init()?;

        tracing::trace!("Setup complete");

        Ok(())
    }
}
