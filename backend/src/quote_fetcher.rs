//! Fetch attestation quote from GCP confidential VM metadata.
//! Returns raw quote bytes; the token is a JWT and the TDX quote is in the payload.
//!
//! **Day 1 on a GCP TDX VM:** Confirm the actual JWT payload field names by running:
//!   curl -H "Metadata-Flavor: Google" \
//!     "http://metadata.google.internal/computeMetadata/v1/instance/confidential-computing/attestation-token?nonce=test123" \
//!     | cut -d. -f2 | base64 -d 2>/dev/null | python3 -m json.tool | head -50
//! Then update the payload struct and extraction below to match (e.g. `eat_claims`, `secboot`, or nested quote location).

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::Deserialize;
use std::fmt;

const GCP_METADATA_URL: &str = "http://metadata.google.internal/computeMetadata/v1/instance/confidential-computing/attestation-token";

/// Error when fetching or decoding a GCP attestation token.
#[derive(Debug)]
pub enum FetchError {
    Request(String),
    Decode(String),
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FetchError::Request(s) => write!(f, "Request failed: {}", s),
            FetchError::Decode(s) => write!(f, "Decode failed: {}", s),
        }
    }
}

impl std::error::Error for FetchError {}

/// JWT payload from GCP attestation token. Field names may vary by GCP attestation version
/// (e.g. `eat_claims`, `secboot`, `quote`, `raw_quote`). Inspect payload on a real CVM and update.
#[derive(Debug, Deserialize)]
struct JwtPayload {
    #[serde(rename = "quote")]
    quote_b64: Option<String>,
    #[serde(rename = "raw_quote")]
    raw_quote_b64: Option<String>,
    #[serde(flatten)]
    _rest: serde_json::Value,
}

/// Fetch attestation token from GCP metadata, decode JWT, and return raw quote bytes.
/// Only works when running on a GCP confidential VM.
pub async fn fetch_gcp_quote(nonce: &str) -> Result<Vec<u8>, FetchError> {
    let url = format!("{}?nonce={}", GCP_METADATA_URL, nonce);
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("Metadata-Flavor", "Google")
        .send()
        .await
        .map_err(|e| FetchError::Request(e.to_string()))?;

    if !res.status().is_success() {
        return Err(FetchError::Request(format!("HTTP {}", res.status())));
    }

    let token = res.text().await.map_err(|e| FetchError::Request(e.to_string()))?;
    let parts: Vec<&str> = token.splitn(3, '.').collect();
    if parts.len() < 2 {
        return Err(FetchError::Decode("JWT must have at least header.payload".into()));
    }

    let payload_b64 = parts[1];
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload_b64.as_bytes())
        .map_err(|e| FetchError::Decode(e.to_string()))?;
    let payload: JwtPayload = serde_json::from_slice(&payload_bytes)
        .map_err(|e| FetchError::Decode(e.to_string()))?;

    let quote_b64 = payload
        .raw_quote_b64
        .or(payload.quote_b64)
        .ok_or_else(|| FetchError::Decode("JWT payload has no quote/raw_quote; run the curl in this file's doc comment on a CVM to see actual field names (e.g. eat_claims, secboot)".into()))?;

    let quote_bytes = URL_SAFE_NO_PAD
        .decode(quote_b64.as_bytes())
        .or_else(|_| URL_SAFE_NO_PAD.decode(quote_b64.trim()))
        .map_err(|e| FetchError::Decode(e.to_string()))?;

    Ok(quote_bytes)
}
