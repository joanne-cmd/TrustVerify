//! Parser for attestation quotes.
//! Supports:
//! - Intel TDX DCAP format: full header + TD report (TCB SVN, mr_td, report_data, Platform ID)
//! - Simplified mock format: first 16 bytes = Platform ID (for short quotes)
//!
//! DCAP byte layout:
//! - Bytes 0–3:   version + attestation key type
//! - Bytes 4–47:  quote header
//!   - Standard DCAP: Platform ID at bytes 28–43 (user_data)
//!   - GCP TDX: Platform ID in bytes 4–19 (TEE type + QE identifier) when bytes 28–43 are zeroed
//! - Bytes 48+:   TD report body (mr_td at offset 128 from TD start, TCB SVN at offset 222)

use serde::{Serialize, Serializer};
use std::fmt;

fn serialize_hex_16<S>(v: &[u8; 16], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&hex::encode(v))
}
fn serialize_hex_48<S>(v: &[u8; 48], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&hex::encode(v))
}
fn serialize_hex_64<S>(v: &[u8; 64], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&hex::encode(v))
}

/// Parsed attestation quote with extracted Platform ID and DCAP fields.
#[derive(Debug, Clone, Serialize)]
pub struct ParsedQuote {
    /// Platform ID (16 bytes) as hex string.
    pub platform_id_hex: String,
    /// TCB security version numbers (16 bytes). Zeroed if not DCAP or quote too short.
    #[serde(serialize_with = "serialize_hex_16")]
    pub tcb_svn: [u8; 16],
    /// Measurement of the TD (VM) code, 48 bytes. Zeroed if not DCAP or quote too short.
    #[serde(serialize_with = "serialize_hex_48")]
    pub mr_td: [u8; 48],
    /// Custom report data embedded in quote, 64 bytes. Zeroed if not present.
    #[serde(serialize_with = "serialize_hex_64")]
    pub report_data: [u8; 64],
    /// Raw bytes of the full quote.
    #[serde(skip)]
    pub raw: Vec<u8>,
    /// Whether this was parsed as DCAP format (vs simplified).
    #[serde(skip)]
    pub is_dcap_format: bool,
}

/// Errors that can occur during quote parsing.
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Input is not valid hex.
    InvalidHex,
    /// Quote is too short (need at least 16 bytes for Platform ID).
    TooShort,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidHex => write!(f, "Invalid hex encoding"),
            ParseError::TooShort => write!(f, "Quote too short (need at least 32 hex chars for Platform ID)"),
        }
    }
}

impl std::error::Error for ParseError {}

/// DCAP layout constants (bytes).
const HEADER_MIN_LEN: usize = 48;
const PLATFORM_ID_START: usize = 28;
const PLATFORM_ID_LEN: usize = 16;
const PLATFORM_ID_FALLBACK_START: usize = 4;
const TD_REPORT_START: usize = 48;
const MR_TD_OFFSET_IN_TD: usize = 128;
const TCB_SVN_OFFSET_IN_TD: usize = 222;
const MR_TD_LEN: usize = 48;
const TCB_SVN_LEN: usize = 16;
const REPORT_DATA_LEN: usize = 64;

fn copy_slice<const N: usize>(bytes: &[u8], start: usize) -> [u8; N] {
    let mut out = [0u8; N];
    let end = (start + N).min(bytes.len());
    let len = end.saturating_sub(start).min(N);
    if len > 0 {
        out[..len].copy_from_slice(&bytes[start..start + len]);
    }
    out
}

/// Parse a hex-encoded attestation quote and extract Platform ID and DCAP fields.
///
/// - **DCAP format** (≥96 hex chars / 48 bytes): Intel DCAP header; Platform ID = bytes 28-43.
///   If bytes 28-43 are all zeroes (e.g. GCP TDX), Platform ID falls back to bytes 4-19.
///   If quote is long enough: TCB SVN at TD report offset 222, mr_td at TD report offset 128.
/// - **Simplified format** (32+ hex chars): First 16 bytes = Platform ID; TCB/mr_td/report_data zeroed.
/// Accepts input with or without "0x" prefix; strips whitespace.
pub fn parse_quote(input: &str) -> Result<ParsedQuote, ParseError> {
    let input = input.trim().trim_start_matches("0x");
    let input: String = input.chars().filter(|c| !c.is_whitespace()).collect();

    if input.is_empty() {
        return Err(ParseError::InvalidHex);
    }

    if input.len() % 2 != 0 || !input.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ParseError::InvalidHex);
    }

    if input.len() < 32 {
        return Err(ParseError::TooShort);
    }

    let bytes = hex::decode(&input).map_err(|_| ParseError::InvalidHex)?;

    let (platform_id_hex, is_dcap_format, tcb_svn, mr_td, report_data) = if bytes.len() >= HEADER_MIN_LEN {
        // Standard DCAP: Platform ID is at bytes 28..44 (user_data).
        // Some providers (notably GCP TDX) zero that region and instead place the
        // identifier in bytes 4..20 (TEE type + QE identifier).
        let primary_platform_id = copy_slice::<PLATFORM_ID_LEN>(&bytes, PLATFORM_ID_START);
        let platform_id = if primary_platform_id.iter().all(|&b| b == 0) {
            copy_slice::<PLATFORM_ID_LEN>(&bytes, PLATFORM_ID_FALLBACK_START)
        } else {
            primary_platform_id
        };
        let platform_id_hex = hex::encode(platform_id);

        let tcb_svn = if bytes.len() >= TD_REPORT_START + TCB_SVN_OFFSET_IN_TD + TCB_SVN_LEN {
            copy_slice::<16>(&bytes, TD_REPORT_START + TCB_SVN_OFFSET_IN_TD)
        } else {
            [0u8; 16]
        };

        let mr_td = if bytes.len() >= TD_REPORT_START + MR_TD_OFFSET_IN_TD + MR_TD_LEN {
            copy_slice::<48>(&bytes, TD_REPORT_START + MR_TD_OFFSET_IN_TD)
        } else {
            [0u8; 48]
        };

        let report_data = if bytes.len() >= TD_REPORT_START + MR_TD_OFFSET_IN_TD + MR_TD_LEN + REPORT_DATA_LEN {
            copy_slice::<64>(&bytes, TD_REPORT_START + MR_TD_OFFSET_IN_TD + MR_TD_LEN)
        } else {
            [0u8; 64]
        };

        (platform_id_hex, true, tcb_svn, mr_td, report_data)
    } else {
        (
            input[..32].to_lowercase(),
            false,
            [0u8; 16],
            [0u8; 48],
            [0u8; 64],
        )
    };

    Ok(ParsedQuote {
        platform_id_hex,
        tcb_svn,
        mr_td,
        report_data,
        raw: bytes,
        is_dcap_format,
    })
}

/// Hex-encode TCB SVN for storage/display.
pub fn tcb_svn_to_hex(svn: &[u8; 16]) -> String {
    hex::encode(svn)
}

/// Hex-encode mr_td for storage/display.
pub fn mr_td_to_hex(mr: &[u8; 48]) -> String {
    hex::encode(mr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_trusted_quote() {
        let quote = "a1b2c3d4e5f6789012345678abcdef0100000000000000000000000000000000000000000000000000000000000000";
        let parsed = parse_quote(quote).unwrap();
        assert_eq!(parsed.platform_id_hex, "a1b2c3d4e5f6789012345678abcdef01");
    }

    #[test]
    fn parse_with_0x_prefix() {
        let quote = "0xa1b2c3d4e5f6789012345678abcdef01000000000000000000000000000000";
        let parsed = parse_quote(quote).unwrap();
        assert_eq!(parsed.platform_id_hex, "a1b2c3d4e5f6789012345678abcdef01");
    }

    #[test]
    fn parse_too_short_fails() {
        let quote = "deadbeef";
        assert!(matches!(parse_quote(quote), Err(ParseError::TooShort)));
    }

    #[test]
    fn parse_invalid_hex_fails() {
        let quote = "notvalidhex!!!";
        assert!(matches!(parse_quote(quote), Err(ParseError::InvalidHex)));
    }

    #[test]
    fn parse_dcap_format_extracts_platform_id_from_user_data() {
        let quote = "030002008100000000000000939a7233f79c4ca9940a0db3957f3465aabbccdd11223344aabbccdd112233440000".to_string()
            + &"00".repeat(100);
        let parsed = parse_quote(&quote).unwrap();
        assert_eq!(parsed.platform_id_hex, "aabbccdd11223344aabbccdd11223344");
        assert!(parsed.is_dcap_format);
    }

    #[test]
    fn parse_dcap_format_falls_back_when_platform_id_zeroed() {
        // Create a "DCAP-shaped" quote long enough to trigger the DCAP branch,
        // with primary Platform ID (bytes 28..44) zeroed and fallback bytes 4..20 populated.
        let mut bytes = vec![0u8; 64];

        // bytes 28..44 = all zeros (primary)
        assert!(bytes[PLATFORM_ID_START..(PLATFORM_ID_START + PLATFORM_ID_LEN)]
            .iter()
            .all(|&b| b == 0));

        let fallback: [u8; PLATFORM_ID_LEN] = [
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff, 0x00,
        ];

        bytes[PLATFORM_ID_FALLBACK_START..(PLATFORM_ID_FALLBACK_START + PLATFORM_ID_LEN)]
            .copy_from_slice(&fallback);

        let quote = hex::encode(&bytes);
        let parsed = parse_quote(&quote).unwrap();

        assert!(parsed.is_dcap_format);
        assert_eq!(parsed.platform_id_hex, hex::encode(&fallback));
    }
}
