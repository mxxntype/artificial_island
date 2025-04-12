use clap::Parser;
use clap::builder::Styles;
use clap::builder::styling::AnsiColor::{BrightMagenta, Green, White};
use std::path::PathBuf;

const STYLES: Styles = Styles::styled()
    .usage(Green.on_default().bold())
    .literal(BrightMagenta.on_default().bold())
    .placeholder(White.on_default().bold())
    .header(Green.on_default().bold());

#[derive(Parser, Debug)]
#[command(version, author, about, styles(STYLES))]
pub struct Options {
    /// Path to the file that will be served by the server.
    pub file: PathBuf,
}
