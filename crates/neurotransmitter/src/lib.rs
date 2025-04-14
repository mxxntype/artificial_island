use epicentre_diagnostics::tracing::{self, instrument};
use iroh::SecretKey;
use std::{env, fs, io, path};

pub mod cli;

#[derive(Debug)]
#[must_use]
pub struct Identity;

impl Identity {
    const PRIVATE_KEY_HOME_LOCATION: &str = ".cache/neurotransmitter/";
    const PRIVATE_KEY_FILENAME: &str = "identity.bin";

    /// Retrieve a cached identity or create & cache a new one.
    ///
    /// # Errors
    ///
    /// This function will attempt to generate and save a new identity if the
    /// cached one can't be reached, so an error is returned only if the cached
    /// identity can't be read **and** a new one cannot be persisted.
    #[instrument(name = "identity_provider", skip_all)]
    pub fn from_cache_or_generate_new(self) -> Result<SecretKey, IdentityError> {
        let home_dir_str = env::var("HOME")?;
        let parent_dir_str = format!("{home_dir_str}/{}", Self::PRIVATE_KEY_HOME_LOCATION);
        let parent_path = path::absolute(parent_dir_str)?;
        let _ = fs::create_dir_all(&parent_path);
        let mut identity_path = parent_path;
        identity_path.push(Self::PRIVATE_KEY_FILENAME);
        let read_from_cache = || -> Result<_, IdentityError> {
            let identity_bytes: [u8; 32] = fs::read(&identity_path)?
                .try_into()
                .map_err(|_| IdentityError::MalformedBytes)?;
            Ok(SecretKey::from_bytes(&identity_bytes))
        };
        read_from_cache().or_else(|error| {
            tracing::debug!(?error, "Failed to read cached identity, generating new one");
            let key = SecretKey::generate(rand::rngs::OsRng);
            fs::write(identity_path, key.to_bytes())?;
            Ok(key)
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum IdentityError {
    EnvVar(#[from] env::VarError),
    Io(#[from] io::Error),
    #[error("The identity file contained malformed bytes")]
    MalformedBytes,
}
