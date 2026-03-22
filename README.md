# TrustVerify

> A reference implementation of PPID-based provider binding for Confidential Virtual Machines, directly implementing the proposals in ["Narrowing the Gap between TEEs Threat Model and Deployment Strategies"](https://arxiv.org/abs/2506.14964) (Rezabek et al., SysTEX '25 @ EuroS&P).

---

## The Problem

When you run a workload inside a cloud Confidential VM (CVM), you can verify *what* is running via a TEE attestation quote. But you cannot verify *where* it is running. The attestation report does not bind the CVM to a specific cloud provider's infrastructure — meaning a user has no cryptographic proof their VM is hosted on legitimate hardware rather than an untrusted machine.

This is the core finding of the IC3 paper this project implements:

> *"Remote attestation verifies that communication terminates in a TEE on an authenticated platform but does not provide details about the operational environment."*

The paper proposes two discussion points (DPs):

- **DP1** — Bind attestation quotes to a known cloud provider via the Protected Platform Identifier (PPID)
- **DP2** — Extend the threat model to include physical access and side-channel attacks

TrustVerify implements DP1 in full and provides indirect signaling for DP2.

---

## What We Built

A working Rust backend + Next.js frontend that:

1. Parses real Intel TDX DCAP attestation quotes from hardware
2. Extracts the Platform ID (PPID) and binds it to a cloud provider registry
3. Verifies registry entries are cryptographically signed (ed25519) — so the registry itself cannot be spoofed
4. Detects TCB security version regressions across quote history (indirect DP2 signal)
5. Detects host migration — PPID changes meaning the VM moved to different physical hardware
6. Returns a composite trust score (0–100) surfacing all signals in a live dashboard

### Verified on real hardware

This project was tested against a real attestation quote from a **GCP TDX Confidential VM** (`c3-standard-4`, `us-central1-a`). The sample quote is included in `samples/real_gcp_tdx.hex`.

---

## Research → Code Mapping

### DP1: PPID-based provider binding

| Paper proposal | Implementation |
|---|---|
| Extract PPID from attestation quote | `backend/src/parser.rs` — parses DCAP quote bytes, extracts Platform ID |
| Maintain provider registry of known PPIDs | `backend/src/registry.rs` — JSON registry with ed25519-signed entries |
| Provider signs hardware manifests | `backend/src/bin/registry_sign.rs` — keygen + signing tool |
| Verify quote is bound to known provider | `POST /api/verify` — returns `provider_match` + `registry_sig_valid` |

### DP2: Indirect physical attack signaling

The paper is honest that full DP2 cannot be solved in software — it requires chip-level design changes. We implement the best available software-layer signals:

| Signal | Implementation |
|---|---|
| TCB regression detection | `backend/src/history.rs` — byte-wise SVN comparison flags firmware downgrades |
| Host migration detection | `backend/src/history.rs` — PPID change across quotes = different physical machine |
| Historical quote timeline | `GET /api/history` — full attestation history per PPID |

### Trust Score (0–100)

| Signal | Points |
|---|---|
| Valid attestation quote | +50 |
| Provider found in registry | +25 |
| No TCB regression detected | +15 |
| No host migration detected | +5 |
| Registry signature valid | +5 |

### What remains open

Per the paper's Section 4, closing DP2 fully requires chip-level design changes and industry standardization across Intel TDX, AMD SEV-SNP, and ARM CCA. TrustVerify implements the feasible software layer. The hardware trust anchor remains future work requiring collaboration between chip manufacturers, cloud providers, and standards bodies.

---

## Architecture

```
┌─────────────────────────────────────────┐
│           Next.js Frontend              │
│  Verify tab · History tab · API docs    │
└──────────────┬──────────────────────────┘
               │ POST /api/verify
               │ GET  /api/history
               │ GET  /api/registry
┌──────────────▼──────────────────────────┐
│           Rust Backend (Axum)           │
│                                         │
│  parser.rs      — DCAP quote parsing    │
│  registry.rs    — provider lookup +     │
│                   ed25519 verification  │
│  history.rs     — SQLite quote store,   │
│                   regression/migration  │
│  verifier.rs    — trust score engine    │
│  quote_fetcher  — GCP metadata client   │
└──────────────┬──────────────────────────┘
               │
       signed_registry.json
       quotes.db (SQLite)
```

---

## Project Structure

```
trustverify/
├── backend/
│   ├── src/
│   │   ├── main.rs           # CLI: serve / verify
│   │   ├── api.rs            # HTTP endpoints
│   │   ├── parser.rs         # DCAP quote parsing
│   │   ├── registry.rs       # Provider registry + ed25519
│   │   ├── history.rs        # SQLite history store
│   │   ├── verifier.rs       # Trust score engine
│   │   ├── quote_fetcher.rs  # GCP metadata client
│   │   └── bin/
│   │       └── registry_sign.rs  # Signing tool
│   ├── Cargo.toml
│   └── Dockerfile
├── frontend/
│   └── src/app/page.tsx      # Full dashboard UI
├── samples/
│   ├── real_gcp_tdx.hex      # Real TDX quote from GCP hardware
│   ├── trusted.hex           # Sample trusted quote
│   ├── unknown.hex           # Sample unknown provider quote
│   └── invalid.hex           # Sample invalid quote
├── registry.json             # Provider registry (unsigned)
├── signed_registry.json      # Provider registry (ed25519 signed)
└── README.md
```

---

## Running Locally

### Prerequisites
- Rust 1.82+
- Node.js 18+

### Backend

```bash
cd backend

# Generate a keypair and sign the registry
cargo run --bin registry_sign -- keygen
cargo run --bin registry_sign -- sign ../registry.json <private_key> > ../signed_registry.json

# Start the server
cargo run --bin trustverify -- serve --registry ../signed_registry.json
# API available at http://localhost:8080
```

### Frontend

```bash
cd frontend
npm install
NEXT_PUBLIC_API_URL=http://localhost:8080 npm run dev
# UI available at http://localhost:3000
```

### Verify a real quote

```bash
curl -s -X POST http://localhost:8080/api/verify \
  -H 'Content-Type: application/json' \
  -d "{\"quote\": \"$(cat samples/real_gcp_tdx.hex)\", \"format\": \"intel_dcap\"}" \
  | python3 -m json.tool
```

Expected output on the real GCP TDX quote:
```json
{
  "status": "Trusted",
  "provider_match": {
    "found": true,
    "provider": "Google Cloud Platform",
    "region": "us-central1-a",
    "signature_valid": true
  },
  "trust_score": 95,
  "tcb_regression": false,
  "migration_detected": false,
  "registry_sig_valid": true
}
```

### Sign registry entries

```bash
cd backend

# Generate a provider keypair
cargo run --bin registry_sign -- keygen
# Output:
# private_key: <base64>
# public_key:  <base64>

# Sign all entries in registry.json
cargo run --bin registry_sign -- sign ../registry.json <private_key_base64> > ../signed_registry.json
```

---

## API Reference

### POST /api/verify
Verify an attestation quote and return trust assessment.

**Request:**
```json
{ "quote": "<hex string>", "format": "intel_dcap" }
```

**Response:**
```json
{
  "valid": true,
  "tee_type": "Intel TDX",
  "ppid_extracted": true,
  "platform_id_truncated": "810000000000...",
  "provider_match": {
    "found": true,
    "provider": "Google Cloud Platform",
    "region": "us-central1-a",
    "verification_level": 2,
    "signature_valid": true
  },
  "status": "Trusted",
  "tcb_svn": "0d01080000000000...",
  "tcb_regression": false,
  "migration_detected": false,
  "registry_sig_valid": true,
  "trust_score": 95,
  "timestamp": "2026-03-19T..."
}
```

### GET /api/history?ppid=\<hex\>
Returns quote history for a platform ID with regression and migration events.

### GET /api/registry
Returns all registry entries with signature validity per entry.

---

## Paper Citation

```
@inproceedings{rezabek2025tee,
  title     = {Narrowing the Gap between TEEs Threat Model and Deployment Strategies},
  author    = {Rezabek, Filip and Passerat-Palmbach, Jonathan and Mahhouk, Moe 
               and Erdmann, Frieder and Miller, Andrew},
  booktitle = {8th Edition of the System Software for Trusted Execution (SysTEX '25),
               co-located with EuroS&P '25},
  year      = {2025}
}
```

---

*Built for the IC3 Shape Rotator Hackathon — implementing groundbreaking confidential computing research.*