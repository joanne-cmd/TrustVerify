//! Parser for attestation quotes.
//! Supports:
//! - Intel DCAP format: 48-byte header, Platform ID in user_data (bytes 28-43)
//! - Simplified mock format: first 16 bytes = Platform ID (for short quotes)

use serde::Serialize;
use std::fmt;

/// Parsed attestation quote with extracted Platform ID.
#[derive(Debug, Clone, Serialize)]
pub struct ParsedQuote {
    /// Platform ID (16 bytes) as hex string.
    pub platform_id_hex: String,
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

/// Parse a hex-encoded attestation quote and extract the Platform ID.
///
/// - **DCAP format** (≥96 hex chars / 48 bytes): Intel DCAP header; Platform ID = bytes 28-43 (user_data).
/// - **Simplified format** (32+ hex chars): First 16 bytes = Platform ID.
/// Accepts input with or without "0x" prefix; strips whitespace.
pub fn parse_quote(input: &str) -> Result<ParsedQuote, ParseError> {
    let input = input.trim().trim_start_matches("0x");
    let input: String = input.chars().filter(|c| !c.is_whitespace()).collect();

    if input.is_empty() {
        return Err(ParseError::InvalidHex);
    }

    // Validate hex
    if input.len() % 2 != 0 || !input.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ParseError::InvalidHex);
    }

    // Need at least 32 hex chars (16 bytes) for Platform ID
    if input.len() < 32 {
        return Err(ParseError::TooShort);
    }

    let bytes = hex::decode(&input).map_err(|_| ParseError::InvalidHex)?;

    // DCAP format: 48-byte header, Platform ID in user_data (bytes 28-43)
    let (platform_id_hex, is_dcap_format) = if bytes.len() >= 48 {
        // user_data starts at byte 28; first 16 bytes = Platform ID
        let platform_id = &input[56..88]; // 28*2=56, 16 bytes=32 hex chars
        (platform_id.to_lowercase(), true)
    } else {
        // Simplified format: first 16 bytes = Platform ID
        (input[..32].to_lowercase(), false)
    };

    Ok(ParsedQuote {
        platform_id_hex,
        raw: bytes,
        is_dcap_format,
    })
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
        // DCAP header: 48 bytes, Platform ID at bytes 28-43 (user_data)
        let quote = "030002008100000000000000939a7233f79c4ca9940a0db3957f3465aabbccdd11223344aabbccdd112233440000".to_string()
            + &"00".repeat(100);
        let parsed = parse_quote(&quote).unwrap();
        assert_eq!(parsed.platform_id_hex, "aabbccdd11223344aabbccdd11223344");
        assert!(parsed.is_dcap_format);
    }
}
