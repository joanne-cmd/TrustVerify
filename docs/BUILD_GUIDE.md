# PPID Verification Dashboard — Build Guide

A step-by-step guide to building the project, with clear differentiators from existing solutions.

---

# Part 1: What We Do Differently

## Comparison Matrix

| Feature | Proof of Cloud | Automata DCAP | Intel/AMD Tools | **Our Project** |
|---------|----------------|---------------|-----------------|-----------------|
| Verify attestation quote | ✅ Web form | ❌ | ✅ CLI libs | ✅ Web + API + CLI |
| PPID → Provider lookup | ✅ Alliance registry | ❌ | ❌ | ✅ Local + extensible |
| Public REST API | ❓ Unclear | ❌ | ❌ | ✅ Yes |
| CLI for CI/CD | ❌ | ❌ | ✅ (low-level) | ✅ High-level |
| Open source | ❌ Alliance-run | Partial | ✅ | ✅ Full |
| Self-hostable | ❌ | ❌ (SaaS) | ✅ | ✅ Yes |
| Developer samples | ❌ | ❌ | Limited | ✅ Mock quotes, docs |
| Educational / explainer | Minimal | Minimal | No | ✅ Field-by-field |
| Local mock registry | ❌ | ❌ | ❌ | ✅ For dev/testing |
| Batch verification | ❌ | ❌ | ❌ | ✅ (stretch) |

---

## Our Differentiators (What Makes Us Unique)

### 1. **Developer-First Experience**
- **Proof of Cloud:** Web form only; no public API documented; alliance membership for full access
- **Us:** REST API, CLI tool, mock registry for local dev, sample attestation quotes in repo

### 2. **Fully Open Source & Self-Hostable**
- **Proof of Cloud:** Alliance-governed; registry not self-hostable
- **Us:** Run locally, use your own registry, no gatekeeping

### 3. **Educational Layer**
- **Others:** Assume you know TEEs
- **Us:** Explain each field (PPID, measurements, verification level); link to paper; threat model summary

### 4. **CI/CD Integration**
- **Proof of Cloud:** Manual web verification
- **Us:** `ppid-verify attestation.hex --registry ./registry.json` for pipelines

### 5. **Multi-Registry Support**
- **Proof of Cloud:** Single alliance registry
- **Us:** Local JSON registry by default; pluggable to Proof of Cloud API when available

### 6. **Complementary to Automata**
- **Automata:** Collateral management (certs, TCB)
- **Us:** PPID + provider binding — we add the layer Automata doesn’t cover

---

# Part 2: Project Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         PPID Verification Dashboard                      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────────────┐  │
│  │   Web UI    │    │   REST API  │    │   CLI (ppid-verify)         │  │
│  │   (React)   │    │   /verify   │    │   For CI/CD, scripts        │  │
│  └──────┬──────┘    └──────┬──────┘    └──────────────┬──────────────┘  │
│         │                  │                          │                  │
│         └──────────────────┼──────────────────────────┘                │
│                            │                                             │
│                            ▼                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐│
│  │                     Verification Engine (Rust)                      ││
│  │  • Parse quote (Intel DCAP / AMD SEV-SNP)                           ││
│  │  • Verify signature (or simplified for demo)                         ││
│  │  • Extract PPID / Platform ID                                       ││
│  │  • Registry lookup                                                  ││
│  └─────────────────────────────────────────────────────────────────────┘│
│                            │                                             │
│                            ▼                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐│
│  │                     Registry (Pluggable)                              ││
│  │  • Local: JSON file (default)                                        ││
│  │  • Future: Proof of Cloud API, on-chain                              ││
│  └─────────────────────────────────────────────────────────────────────┘│
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

# Part 3: Step-by-Step Build Guide

## Phase 1: Foundation (Days 1–2)

### Step 1.1: Project Setup
```bash
# Create workspace
mkdir trustverify && cd trustverify

# Rust backend
cargo init --name trustverify

# Frontend (optional: do after backend works)
npx create-next-app@latest frontend --typescript --tailwind --app
```

### Step 1.2: Get Sample Attestation Quotes
- **Intel DCAP:** Use samples from Intel docs or Cosmian `intel-sgx-ra` test fixtures
- **Mock:** Create a minimal binary structure with a known "PPID" for testing
- **Reference:** https://github.com/intel/SGXDataCenterAttestationPrimitives (sample apps)

### Step 1.3: Parse Quote Structure
- **Intel DCAP:** Quote format is documented in Intel TDX DCAP API PDF
- **Key fields:** `report_data` (user data), first 16 bytes often used for Platform ID
- **Libraries:** `sgx-dcap-ql` (C, via bindings) or Cosmian `intel-sgx-ra` (Python) — or parse manually for demo

### Step 1.4: Create Mock Registry
```json
// registry.json
{
  "version": "1.0",
  "entries": [
    {
      "platform_id_hex": "a1b2c3d4e5f6...",
      "provider": "Demo AWS",
      "region": "us-east-1",
      "verification_level": 1
    }
  ]
}
```

---

## Phase 2: Core Logic (Days 3–4)

### Step 2.1: Verification Engine
Implement in Rust:
1. Decode hex/base64 input
2. Parse quote header → detect format (Intel vs AMD)
3. Extract Platform ID / PPID from correct offset
4. Look up in registry
5. Return structured result

### Step 2.2: REST API
```
POST /api/verify
Content-Type: application/json
{ "quote": "0x...", "format": "intel_dcap" }

Response:
{
  "valid": true,
  "tee_type": "Intel TDX",
  "platform_id_truncated": "a1b2c3...",
  "provider": { "name": "Demo AWS", "region": "us-east-1" },
  "verification_level": 1
}
```

### Step 2.3: CLI
```bash
ppid-verify --quote attestation.hex --registry registry.json
# Exit 0 = trusted, 1 = unknown, 2 = invalid
```

---

## Phase 3: Frontend (Days 4–5)

### Step 3.1: Input Screen
- Textarea for quote (hex or base64)
- Format selector: Intel DCAP | AMD SEV-SNP | Auto
- "Verify" button
- Optional: File upload

### Step 3.2: Results Screen
- Status badge: Trusted (green) | Unknown (yellow) | Invalid (red)
- TEE type, truncated PPID
- Provider name + region
- Collapsible "Learn more" with field explanations

### Step 3.3: Educational Section
- "What is PPID?" — from the paper
- "Why does provider matter?" — threat model gap
- Link to Proof of Cloud, paper, docs

---

## Phase 4: Polish & Differentiators (Days 6–7)

### Step 4.1: Add CLI
- `cargo install` for `ppid-verify`
- Document in README for CI/CD use

### Step 4.2: Batch Endpoint (Stretch)
```
POST /api/verify/batch
{ "quotes": ["0x...", "0x..."] }
```

### Step 4.3: Proof of Cloud Integration (Stretch)
- If PoC exposes public API: add registry adapter
- Document: "Use local registry for dev; Proof of Cloud for production"

### Step 4.4: Documentation
- README with quickstart
- API docs (OpenAPI/Swagger)
- Demo script for judges

---

# Part 4: File Structure

```
trustverify/
├── docs/                    # All documentation
├── README.md
├── registry.json            # Mock registry
├── samples/                 # Sample attestation quotes
│   ├── trusted.hex
│   ├── unknown.hex
│   └── README.md
├── backend/                 # Rust
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── parser.rs
│   │   ├── verifier.rs
│   │   ├── registry.rs
│   │   └── api.rs
│   └── ...
├── frontend/                # Next.js
│   ├── app/
│   │   ├── page.tsx
│   │   └── api/...
│   └── ...
└── cli/                     # Or same crate as backend
    └── src/main.rs
```

---

# Part 5: Success Checklist

Before demo, ensure:

- [ ] Paste sample quote → get "Trusted" or "Unknown" correctly
- [ ] REST API returns valid JSON
- [ ] CLI works: `ppid-verify quote.hex`
- [ ] UI explains what PPID and provider mean
- [ ] README has quickstart
- [ ] Differentiators documented (vs Proof of Cloud, Automata)

---

# Part 6: Pitch Talking Points

1. **"Proof of Cloud exists, but we built an open-source, developer-first alternative."**
2. **"You can self-host, use a local registry for testing, and integrate into CI/CD."**
3. **"We add the PPID → provider layer that Automata's collateral dashboard doesn't cover."**
4. **"Our tool is educational — we explain the threat model gap from the paper."**

---

*Build Guide v1.0 — March 2025*
