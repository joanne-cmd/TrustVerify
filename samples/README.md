# Sample Attestation Quotes

These are **mock quotes** in a simplified format for hackathon testing. See [SAMPLE_QUOTES_GUIDE.md](../docs/SAMPLE_QUOTES_GUIDE.md) for details.

## Format (Simplified for Demo)

- **First 16 bytes (32 hex chars)** = Platform ID
- **Remaining bytes** = padding (ignored for lookup)

## Files

| File | Platform ID | Expected Result |
|------|-------------|-----------------|
| `trusted.hex` | `a1b2c3d4e5f6789012345678abcdef01` | **Trusted** — in registry (Demo AWS) |
| `unknown.hex` | `ffffffffffffffffffffffffffffffff` | **Unknown** — not in registry |
| `invalid.hex` | N/A (malformed) | **Invalid** — parse error |

## Registry

The Platform ID from `trusted.hex` is in `../registry.json` as "Demo AWS".
