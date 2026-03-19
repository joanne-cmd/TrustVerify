# Where to Get Sample Attestation Quotes

You need 3 types of quotes for testing. Here are your options.

---

## Option 1: Use a Simplified Mock Format (Recommended for Hackathon)

**You don't need real TEE hardware.** Define a simple format your parser understands and create mock files.

### How It Works

1. **Define a minimal "quote" format** — e.g., first 16 bytes = Platform ID, rest is padding
2. **Create the files** — hex strings you control
3. **Build your parser** to extract the Platform ID from your format
4. **Registry** — add that Platform ID (or its hash) to `registry.json`

### Example: Simple Mock Format

```
[16 bytes Platform ID][optional padding]
```

**trusted.hex** — Platform ID = `a1b2c3d4e5f6789012345678abcdef01` (in registry)
```
a1b2c3d4e5f6789012345678abcdef01000000000000000000000000000000
```

**unknown.hex** — Platform ID = `ffffffffffffffffffffffffffffffff` (NOT in registry)
```
ffffffffffffffffffffffffffffffff000000000000000000000000000000
```

**invalid.hex** — Not valid hex or wrong length
```
deadbeef
```

### Registry Entry for Trusted

```json
{
  "entries": [
    {
      "platform_id_hex": "a1b2c3d4e5f6789012345678abcdef01",
      "provider": "Demo AWS",
      "region": "us-east-1"
    }
  ]
}
```

**Pros:** Full control, no dependencies, works immediately  
**Cons:** Not real Intel/AMD format — document this for judges

---

## Option 2: Use `tdx-quote` Crate (Rust) — Mock Feature

The **tdx-quote** crate has a `mock` feature for generating mock TDX quotes without hardware.

```toml
# Cargo.toml
[dependencies]
tdx-quote = { version = "0.1", features = ["mock"] }
```

```rust
// Generate mock quote for testing
use tdx_quote::Quote;
let mock_quote = Quote::mock(); // or similar API - check crate docs
```

**Repo:** https://github.com/entropyxyz/tdx-quote  
**Note:** Check the crate's API for how to generate mocks and extract Platform ID.

---

## Option 3: Use `calimero-tee-attestation` (Rust)

Platform-agnostic crate that **auto-generates mock attestations** on non-TEE platforms.

```toml
[dependencies]
calimero-tee-attestation = "0.x"
```

- On non-Linux / non-TEE: generates mock attestations automatically
- Has `verify_mock_attestation()` for testing

**Docs:** https://docs.rs/calimero-tee-attestation/

---

## Option 4: Intel DCAP Sample Collateral (Advanced)

Intel's Quote Verification Library includes sample quotes and collateral for testing.

- **Repo:** https://github.com/intel/sgx-tdx-dcap-quoteverificationlibrary
- **Path:** Look for `SampleApp`, `tests/`, or `sample_quote` in the repo
- **Caveat:** Requires building from source; collateral may expire; complex setup

---

## Option 5: Cosmian `intel-sgx-ra` (Python)

Python library for Intel SGX DCAP. May have test fixtures.

- **Repo:** https://github.com/Cosmian/intel-sgx-ra
- Check `tests/` or `examples/` for sample quotes

---

## Recommendation for Your 5-Day Build

| Day | Approach |
|-----|----------|
| **Day 1** | Use **Option 1** (simplified mock format). Create `trusted.hex`, `unknown.hex`, `invalid.hex` by hand. Get the full flow working. |
| **Day 2–5** | Keep mock format. Document: *"We use a simplified format for demo; production would use real Intel DCAP / AMD SEV-SNP quotes."* |
| **Stretch** | If time allows, integrate `tdx-quote` or `calimero-tee-attestation` for real-format mock quotes. |

---

## Quick Start: Create Mock Files Now

Run this to create the sample files:

```bash
cd /home/joanne/ppid-verification-dashboard
mkdir -p samples

# Trusted — Platform ID that we'll add to registry
echo "a1b2c3d4e5f6789012345678abcdef0100000000000000000000000000000000000000000000000000000000000000" > samples/trusted.hex

# Unknown — Platform ID NOT in registry  
echo "ffffffffffffffffffffffffffffffff0000000000000000000000000000000000000000000000000000000000000000" > samples/unknown.hex

# Invalid
echo "deadbeef" > samples/invalid.hex
```

Then in your parser, for the "simplified" format:
- Read hex, decode to bytes
- First 16 bytes = Platform ID
- Look up in registry by `platform_id_hex` or `sha256(platform_id)`

---

## For Your Pitch

> "We use a simplified mock quote format for the hackathon demo because we don't have access to Intel TDX or AMD SEV-SNP hardware. The flow — parse, extract Platform ID, registry lookup — is identical. In production, we'd integrate with real DCAP/SEV-SNP parsers."

Judges will understand. The paper's authors (Flashbots) know real quotes require real hardware.
