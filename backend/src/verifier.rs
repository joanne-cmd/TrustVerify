//! Verification engine: parse quote → extract Platform ID → registry lookup → optional history.

use crate::history::HistoryStore;
use crate::parser::{parse_quote, tcb_svn_to_hex, mr_td_to_hex};
use crate::registry::{load_registry, lookup_provider, ProviderMatch, Registry};
use serde::Serialize;
use std::path::Path;

/// Full verification result returned to API/CLI.
#[derive(Debug, Clone, Serialize)]
pub struct VerificationResult {
    pub valid: bool,
    pub tee_type: String,
    pub ppid_extracted: bool,
    pub platform_id_truncated: Option<String>,
    pub provider_match: ProviderMatch,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub timestamp: String,
    /// TCB SVN hex (from quote).
    pub tcb_svn: String,
    /// True if a previously seen TCB SVN was higher (regression).
    pub tcb_regression: bool,
    /// True if last stored quote had a different PPID (migration).
    pub migration_detected: bool,
    /// True if registry entry had no signature or signature verified.
    pub registry_sig_valid: bool,
    /// Composite trust score 0–100.
    pub trust_score: u8,
}

impl VerificationResult {
    fn build(
        valid: bool,
        tee_type: &str,
        ppid_extracted: bool,
        platform_id_truncated: Option<String>,
        provider_match: ProviderMatch,
        status: &str,
        error: Option<String>,
        tcb_svn: String,
        tcb_regression: bool,
        migration_detected: bool,
        registry_sig_valid: bool,
        trust_score: u8,
    ) -> Self {
        Self {
            valid,
            tee_type: tee_type.to_string(),
            ppid_extracted,
            platform_id_truncated,
            provider_match,
            status: status.to_string(),
            error,
            timestamp: chrono_timestamp(),
            tcb_svn,
            tcb_regression,
            migration_detected,
            registry_sig_valid,
            trust_score,
        }
    }

    fn trusted(
        provider_match: ProviderMatch,
        platform_id_truncated: Option<String>,
        tcb_svn: String,
        tcb_regression: bool,
        migration_detected: bool,
        registry_sig_valid: bool,
        trust_score: u8,
    ) -> Self {
        Self::build(
            true,
            "Intel TDX",
            true,
            platform_id_truncated,
            provider_match,
            "Trusted",
            None,
            tcb_svn,
            tcb_regression,
            migration_detected,
            registry_sig_valid,
            trust_score,
        )
    }

    fn unknown(
        platform_id_truncated: Option<String>,
        tcb_svn: String,
        tcb_regression: bool,
        migration_detected: bool,
        trust_score: u8,
    ) -> Self {
        Self::build(
            true,
            "Intel TDX",
            true,
            platform_id_truncated,
            ProviderMatch {
                found: false,
                provider: None,
                region: None,
                verification_level: None,
                signature_valid: None,
            },
            "Unknown",
            None,
            tcb_svn,
            tcb_regression,
            migration_detected,
            true,
            trust_score,
        )
    }

    fn invalid(error: String) -> Self {
        Self::build(
            false,
            "Unknown",
            false,
            None,
            ProviderMatch {
                found: false,
                provider: None,
                region: None,
                verification_level: None,
                signature_valid: None,
            },
            "Invalid",
            Some(error),
            String::new(),
            false,
            false,
            true,
            0,
        )
    }
}

fn chrono_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn truncate_platform_id(hex: &str) -> String {
    if hex.len() <= 12 {
        hex.to_string()
    } else {
        format!("{}...", &hex[..12])
    }
}

/// Compute trust score 0–100 from verification state.
fn compute_trust_score(
    valid: bool,
    provider_found: bool,
    tcb_regression: bool,
    migration_detected: bool,
    registry_sig_valid: bool,
) -> u8 {
    if !valid {
        return 0;
    }
    let mut score: u8 = 50; // base for valid quote
    if provider_found {
        score = score.saturating_add(25);
    }
    if !tcb_regression {
        score = score.saturating_add(15);
    }
    if !migration_detected {
        score = score.saturating_add(5);
    }
    if registry_sig_valid {
        score = score.saturating_add(5);
    }
    score.min(100)
}

/// Verify a quote against the registry (loads registry from path).
pub fn verify(quote: &str, registry_path: impl AsRef<Path>) -> VerificationResult {
    let registry = match load_registry(registry_path) {
        Ok(r) => r,
        Err(e) => {
            return VerificationResult::invalid(format!("Failed to load registry: {}", e));
        }
    };
    verify_with_registry(quote, &registry, None)
}

/// Verify a quote using an already-loaded registry and optional history store.
pub fn verify_with_registry(
    quote: &str,
    registry: &Registry,
    history: Option<&HistoryStore>,
) -> VerificationResult {
    let parsed = match parse_quote(quote) {
        Ok(p) => p,
        Err(e) => {
            return VerificationResult::invalid(e.to_string());
        }
    };

    let tcb_svn_hex = tcb_svn_to_hex(&parsed.tcb_svn);
    let mr_td_hex = mr_td_to_hex(&parsed.mr_td);
    let truncated = Some(truncate_platform_id(&parsed.platform_id_hex));
    let provider_match = lookup_provider(registry, &parsed.platform_id_hex);

    let tcb_regression = history
        .and_then(|h| h.detect_regression(&parsed.platform_id_hex, &tcb_svn_hex).ok().flatten())
        .is_some();
    let migration_detected = history
        .and_then(|h| h.detect_migration(&parsed.platform_id_hex))
        .is_some();

    let registry_sig_valid = provider_match
        .signature_valid
        .unwrap_or(true);

    if let Some(h) = history {
        let record = crate::history::QuoteRecord {
            ppid: parsed.platform_id_hex.clone(),
            tcb_svn: tcb_svn_hex.clone(),
            mr_td: mr_td_hex,
            timestamp: chrono_timestamp(),
            provider: provider_match.provider.clone(),
        };
        let _ = h.insert(&record);
    }

    let trust_score = compute_trust_score(
        true,
        provider_match.found,
        tcb_regression,
        migration_detected,
        registry_sig_valid,
    );

    if provider_match.found {
        VerificationResult::trusted(
            provider_match,
            truncated,
            tcb_svn_hex,
            tcb_regression,
            migration_detected,
            registry_sig_valid,
            trust_score,
        )
    } else {
        VerificationResult::unknown(
            truncated,
            tcb_svn_hex,
            tcb_regression,
            migration_detected,
            trust_score,
        )
    }
}
