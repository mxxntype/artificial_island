use epicentre_diagnostics::{DiagnosticLayer, Report};
use mpd_android_playlist_bridge::{RawSettings, Settings};

fn main() -> Result<(), Report> {
    DiagnosticLayer.setup()?;

    let raw_settings = RawSettings::default();
    let settings = Settings::try_from(raw_settings)?;
    mpd_android_playlist_bridge::run(&settings)?;

    Ok(())
}
