//! Provider registry for PPID/Platform ID lookup.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Registry of known providers by Platform ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    pub entries: Vec<RegistryEntry>,
}

/// A single registry entry mapping Platform ID to provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub platform_id_hex: String,
    pub provider: String,
    pub region: String,
    #[serde(default)]
    pub verification_level: u8,
    #[serde(default)]
    pub added_at: Option<String>,
}

/// Result of a registry lookup.
#[derive(Debug, Clone, Serialize)]
pub struct ProviderMatch {
    pub found: bool,
    pub provider: Option<String>,
    pub region: Option<String>,
    pub verification_level: Option<u8>,
}

/// Load registry from a JSON file.
pub fn load_registry(path: impl AsRef<Path>) -> Result<Registry, Box<dyn std::error::Error + Send + Sync>> {
    let contents = std::fs::read_to_string(path)?;
    let registry: Registry = serde_json::from_str(&contents)?;
    Ok(registry)
}

/// Look up a Platform ID (hex) in the registry.
/// Returns provider info if found, None otherwise.
pub fn lookup_provider(registry: &Registry, platform_id_hex: &str) -> ProviderMatch {
    let normalized = platform_id_hex.trim().to_lowercase();

    for entry in &registry.entries {
        if entry.platform_id_hex.trim().to_lowercase() == normalized {
            return ProviderMatch {
                found: true,
                provider: Some(entry.provider.clone()),
                region: Some(entry.region.clone()),
                verification_level: Some(entry.verification_level),
            };
        }
    }

    ProviderMatch {
        found: false,
        provider: None,
        region: None,
        verification_level: None,
    }
}
