//! Provider registry for PPID/Platform ID lookup and optional entry signatures.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Canonical message used for ed25519 verification of a registry entry.
fn entry_signed_message(entry: &RegistryEntry) -> Vec<u8> {
    let s = format!(
        "{}|{}|{}|{}",
        entry.platform_id_hex.trim().to_lowercase(),
        entry.provider,
        entry.region,
        entry.verification_level
    );
    s.into_bytes()
}

/// Verify an entry's optional ed25519 signature. Returns true if no signature, or signature is valid.
pub fn verify_entry_signature(entry: &RegistryEntry) -> bool {
    let (Some(sig_b64), Some(pubkey_b64)) = (entry.signature.as_ref(), entry.signer_pubkey.as_ref()) else {
        return true;
    };
    use base64::{engine::general_purpose::STANDARD, Engine};
    let sig_bytes = match STANDARD.decode(sig_b64.as_bytes()) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let pubkey_bytes = match STANDARD.decode(pubkey_b64.as_bytes()) {
        Ok(b) => b,
        Err(_) => return false,
    };
    if sig_bytes.len() != 64 || pubkey_bytes.len() != 32 {
        return false;
    }
    let message = entry_signed_message(entry);
    let pk = ring::signature::UnparsedPublicKey::new(
        &ring::signature::ED25519,
        pubkey_bytes.as_slice(),
    );
    pk.verify(message.as_slice(), sig_bytes.as_slice()).is_ok()
}

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
    #[serde(default)]
    pub signature: Option<String>,
    #[serde(default)]
    pub signer_pubkey: Option<String>,
}

/// Result of a registry lookup.
#[derive(Debug, Clone, Serialize)]
pub struct ProviderMatch {
    pub found: bool,
    pub provider: Option<String>,
    pub region: Option<String>,
    pub verification_level: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_valid: Option<bool>,
}

/// Load registry from a JSON file.
pub fn load_registry(path: impl AsRef<Path>) -> Result<Registry, Box<dyn std::error::Error + Send + Sync>> {
    let contents = std::fs::read_to_string(path)?;
    let registry: Registry = serde_json::from_str(&contents)?;
    Ok(registry)
}

/// Look up a Platform ID (hex) in the registry.
/// Returns provider info if found; signature_valid set when entry has signature/signer_pubkey.
pub fn lookup_provider(registry: &Registry, platform_id_hex: &str) -> ProviderMatch {
    let normalized = platform_id_hex.trim().to_lowercase();

    for entry in &registry.entries {
        if entry.platform_id_hex.trim().to_lowercase() == normalized {
            let signature_valid = (entry.signature.is_some() || entry.signer_pubkey.is_some())
                .then(|| verify_entry_signature(entry));
            return ProviderMatch {
                found: true,
                provider: Some(entry.provider.clone()),
                region: Some(entry.region.clone()),
                verification_level: Some(entry.verification_level),
                signature_valid,
            };
        }
    }

    ProviderMatch {
        found: false,
        provider: None,
        region: None,
        verification_level: None,
        signature_valid: None,
    }
}
