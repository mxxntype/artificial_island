use std::path::PathBuf;
use std::{fs, io};

use epicentre_diagnostics::tracing;
use serde::Deserialize;

#[tracing::instrument]
pub fn run(
    Settings {
        source_directory,
        target_directory,
        prefix,
    }: &Settings,
) -> io::Result<()> {
    for directory_entry in fs::read_dir(source_directory)? {
        match directory_entry {
            Err(error) => tracing::error!(?error),
            Ok(directory_entry) => {
                let file_path = directory_entry.path().canonicalize()?;
                let is_file = file_path.is_file();
                let is_m3u = file_path
                    .extension()
                    .is_some_and(|extension| extension == "m3u");

                if is_file && is_m3u {
                    tracing::debug!(path = %file_path.display(), "Processing M3U file");

                    let m3u_file_contents = fs::read_to_string(&file_path)?;
                    let m3u_adapted_contents = m3u_file_contents
                        .lines()
                        .map(|track_path| prefix.join(track_path).to_string_lossy().to_string())
                        .collect::<Vec<_>>()
                        .join("\n");

                    let _ = fs::create_dir_all(target_directory);
                    let file_name = file_path.file_name().unwrap();
                    let target_file_path = target_directory.join(file_name);

                    fs::write(&target_file_path, m3u_adapted_contents)
                        .inspect(|()| tracing::info!(target_file_path = %target_file_path.display(), "Wrote adapted M3U"))
                        .inspect_err(|error| tracing::error!(?error))?;
                }
            }
        }
    }

    Ok(())
}

#[derive(Deserialize, Debug)]
#[serde(try_from = "RawSettings")]
pub struct Settings {
    pub source_directory: PathBuf,
    pub target_directory: PathBuf,
    pub prefix: PathBuf,
}

#[derive(Deserialize, Debug)]
pub struct RawSettings {
    pub source_directory: PathBuf,
    pub target_directory: PathBuf,
    pub prefix: PathBuf,
}

impl Default for RawSettings {
    fn default() -> Self {
        let home_dir = std::env::home_dir().unwrap();
        let source_directory = home_dir.join(".local/share/mpd/playlists");
        let target_directory = home_dir.join(".local/share/mpd/android_compatible_playlists");
        let prefix = PathBuf::from("/storage/emulated/0/Music");

        Self {
            source_directory,
            target_directory,
            prefix,
        }
    }
}

impl TryFrom<RawSettings> for Settings {
    type Error = io::Error;

    fn try_from(
        RawSettings {
            source_directory,
            target_directory,
            prefix,
        }: RawSettings,
    ) -> io::Result<Self> {
        let settings = Self {
            source_directory: source_directory.canonicalize()?,
            target_directory,
            prefix,
        };

        Ok(settings)
    }
}
