//!
//! A Rust API for getting chain information from the [Cosmos Chain Registry](https://github.com/cosmos/chain-registry).
//!
//! ## Example
//!
//! ```rust
//! use cosmos_chain_registry::ChainRegistry;
//!
//! let registry = ChainRegistry::from_remote().unwrap();
//! let info = registry.get_by_chain_id("juno-1").unwrap();
//!
//! assert_eq!(info.chain_name, "juno");
//! assert_eq!(info.chain_id, "juno-1");
//! assert_eq!(info.pretty_name, "Juno");
//! ```
//!
pub use chain::ChainInfo;
use git2::FetchOptions;
use lazy_static::lazy_static;
use std::path::PathBuf;
use tracing::{debug, info};

mod chain;

/// Generic error type for this crate
pub type Error = Box<dyn std::error::Error>;

lazy_static! {
    /// The git url for the chain registry to clone. This is the default url, but can be overridden by
    /// setting the `CHAIN_REGISTRY_URL` environment variable.
    pub static ref GITHUB_CHAIN_REGISTRY_URL: String = std::env::var("GITHUB_CHAIN_REGISTRY_URL")
        .unwrap_or_else(|_| { "https://github.com/cosmos/chain-registry".to_string() });

    /// The git ref to checkout. This is the default ref, but can be overridden by setting the
    /// `CHAIN_REGISTRY_REF` environment variable.
    pub static ref GITHUB_CHAIN_REGISTRY_REF: String =
        std::env::var("GITHUB_CHAIN_REGISTRY_REF").unwrap_or_else(|_| { "master".to_string() });
}

/// The `ChainRegistry` struct is used to fetch and parse chain information from the
/// [Cosmos Chain Registry](https://github.com/cosmos/chain-registry).
pub struct ChainRegistry {
    path: PathBuf,
}

impl ChainRegistry {
    /// Creates a new `ChainRegistry` instance. The `path` argument is the path to the
    /// local clone of the [Cosmos Chain Registry](https://github.com/cosmos/chain-registry).
    pub fn from_remote() -> Result<Self, Error> {
        // Store the chain registry in a local hidden directory
        let pwd = std::env::current_dir()?;
        let repo_path = pwd.join(".cosmos-chain-registry");
        info!(
            "Cloning chain registry from {} to {}",
            GITHUB_CHAIN_REGISTRY_URL.as_str(),
            repo_path.display()
        );

        // Try to clone the repo
        match git2::Repository::clone(GITHUB_CHAIN_REGISTRY_URL.as_str(), &repo_path) {
            Err(e) => match e.code() {
                // If the repo already exists, pull the latest changes
                git2::ErrorCode::Exists => {
                    debug!("Chain registry already exists, pulling latest changes");
                    // Get the repo
                    let repo = git2::Repository::open(&repo_path)?;
                    // Get the remote
                    let mut remote = repo.find_remote("origin")?;

                    // Fetch the latest changes
                    let mut fo = FetchOptions::new();
                    remote.fetch(&[GITHUB_CHAIN_REGISTRY_REF.as_str()], Some(&mut fo), None)?;

                    // Checkout the latest changes
                    let (object, reference) = repo.revparse_ext(&GITHUB_CHAIN_REGISTRY_REF)?;
                    repo.checkout_tree(&object, None)?;
                    match reference {
                        Some(gref) => repo.set_head(gref.name().unwrap()),
                        None => repo.set_head_detached(object.id()),
                    }?;
                }
                _ => return Err(e.into()),
            },
            Ok(_) => (),
        };

        let registry = Self { path: repo_path };
        Ok(registry)
    }

    /// Get a chain's information from the registry based on the chain_id.
    /// Returns `None` if the chain_id is not found.
    ///
    /// # Arguments
    ///
    /// `chain_id` - The chain_id of the chain to get information for. This is the `chain_id` field in the chain's `chain.json` file. For example, the `chain_id` for the Cosmos Hub is `cosmoshub-4`.
    pub fn get_by_chain_id(&self, chain_id: &str) -> Result<ChainInfo, Error> {
        for file in glob::glob(&self.path.join("**/chain.json").to_string_lossy())? {
            let file = file?;
            let chain_info: ChainInfo = serde_json::from_reader(std::fs::File::open(file)?)?;

            if chain_info.chain_id == chain_id {
                return Ok(chain_info);
            }
        }

        Err("Chain not found".into())
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;

    #[test]
    #[serial]
    fn can_get_chain_registry_data() {
        let registry = ChainRegistry::from_remote();
        assert!(registry.is_ok());
    }

    #[test]
    #[serial]
    fn can_get_chain_config_by_id() {
        let registry = ChainRegistry::from_remote().unwrap();
        let info = registry.get_by_chain_id("juno-1").unwrap();

        assert_eq!(info.chain_name, "juno");
        assert_eq!(info.chain_id, "juno-1");
        assert_eq!(info.pretty_name, "Juno");

        let registry = ChainRegistry::from_remote().unwrap();
        let info = registry.get_by_chain_id("uni-5").unwrap();

        assert_eq!(info.chain_name, "junotestnet");
        assert_eq!(info.chain_id, "uni-5");
        assert_eq!(info.pretty_name, "Juno Testnet");
    }
}
