//! Verification engine: parse quote → extract Platform ID → registry lookup.

use crate::parser::parse_quote;
use crate::registry::{load_registry, lookup_provider, ProviderMatch, Registry};
use serde::Serialize;
use std::path::Path;

/// Full verification result returned to API/CLI.
#[derive(Debug, Clone, Serialize)]
pub struct VerificationResult {
    /// Whether the quote was parsed successfully.
    pub valid: bool,
    /// TEE type (simplified: we use "Intel DCAP (mock)" for our format).
    pub tee_type: String,
    /// Whether we extracted a Platform ID.
    pub ppid_extracted: bool,
    /// Truncated Platform ID for display (first 12 chars + ...).
    pub platform_id_truncated: Option<String>,
    /// Provider lookup result.
    pub provider_match: ProviderMatch,
    /// Human-readable status: "Trusted", "Unknown", or "Invalid".
    pub status: String,
    /// Error message if parsing failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Timestamp (ISO 8601).
    pub timestamp: String,
}

impl VerificationResult {
    fn trusted(provider_match: ProviderMatch, platform_id_truncated: Option<String>) -> Self {
        Self {
            valid: true,
            tee_type: "Intel DCAP (mock)".to_string(),
            ppid_extracted: true,
            platform_id_truncated,
            provider_match,
            status: "Trusted".to_string(),
            error: None,
            timestamp: chrono_timestamp(),
        }
    }

    fn unknown(platform_id_truncated: Option<String>) -> Self {
        Self {
            valid: true,
            tee_type: "Intel DCAP (mock)".to_string(),
            ppid_extracted: true,
            platform_id_truncated,
            provider_match: ProviderMatch {
                found: false,
                provider: None,
                region: None,
                verification_level: None,
            },
            status: "Unknown".to_string(),
            error: None,
            timestamp: chrono_timestamp(),
        }
    }

    fn invalid(error: String) -> Self {
        Self {
            valid: false,
            tee_type: "Unknown".to_string(),
            ppid_extracted: false,
            platform_id_truncated: None,
            provider_match: ProviderMatch {
                found: false,
                provider: None,
                region: None,
                verification_level: None,
            },
            status: "Invalid".to_string(),
            error: Some(error),
            timestamp: chrono_timestamp(),
        }
    }
}

fn chrono_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Truncate Platform ID for display.
fn truncate_platform_id(hex: &str) -> String {
    if hex.len() <= 12 {
        hex.to_string()
    } else {
        format!("{}...", &hex[..12])
    }
}

/// Verify a quote against the registry.
///
/// # Arguments
/// * `quote` - Hex-encoded attestation quote (with or without 0x prefix)
/// * `registry_path` - Path to registry.json
pub fn verify(quote: &str, registry_path: impl AsRef<Path>) -> VerificationResult {
    let registry = match load_registry(registry_path) {
        Ok(r) => r,
        Err(e) => {
            return VerificationResult::invalid(format!("Failed to load registry: {}", e));
        }
    };

    verify_with_registry(quote, &registry)
}

/// Verify a quote using an already-loaded registry.
pub fn verify_with_registry(quote: &str, registry: &Registry) -> VerificationResult {
    let parsed = match parse_quote(quote) {
        Ok(p) => p,
        Err(e) => {
            return VerificationResult::invalid(e.to_string());
        }
    };

    let truncated = Some(truncate_platform_id(&parsed.platform_id_hex));
    let provider_match = lookup_provider(registry, &parsed.platform_id_hex);

    if provider_match.found {
        VerificationResult::trusted(provider_match, truncated)
    } else {
        VerificationResult::unknown(truncated)
    }
}
