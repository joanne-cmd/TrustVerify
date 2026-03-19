# PPID Verification Dashboard — Project Documentation

## Executive Summary

A web-based tool that parses TEE attestation reports (Intel DCAP, AMD SEV-SNP) and verifies whether the PPID/Platform ID matches a known cloud provider registry, answering: **"Is this confidential workload running on trusted, verified hardware?"**

---

# Part 1: Existing Projects (What Already Exists)

## ✅ Yes — Similar Projects Exist

### 1. **Proof of Cloud** (Closest Match)
**URL:** https://proofofcloud.org/

**What it does:**
- Maintains a **public registry** binding hardware IDs (Intel PPID, AMD Chip ID) to verified physical locations
- Has a **"Verify an Attestation"** feature on the website
- Alliance-based: Multiple orgs (Flashbots, Automata, Phala, Secret Network, etc.) independently verify facilities
- Append-only signed log (Certificate Transparency–style)
- Three verification levels: Human-Assisted, Automated, Continuous Monitoring

**Gap your project could fill:**
- Proof of Cloud is alliance-governed and may not be fully open-source or easily self-hostable
- Your project could be a **simpler, open-source prototype** or **developer-focused dashboard** with better UX
- You could build a **client** that integrates with Proof of Cloud's registry
- Or focus on **local verification** with a mock registry for hackathon demos

---

### 2. **Automata DCAP Dashboard**
**URL:** https://docs.ata.network/dcap-dashboard

**What it does:**
- Manages **collateral lifecycle** (certificates, TCB info, CRLs) for Intel SGX/TDX quote verification
- Ensures trust chain stays valid as certs expire

**What it does NOT do:**
- Does NOT verify PPID against provider registry
- Does NOT answer "which provider hosts this TEE?"

**Gap your project fills:**
- PPID verification is a different layer — Automata handles collateral; you handle provider binding

---

### 3. **Intel DCAP / AMD SEV Tools**
- **Intel:** `sgx-dcap-quoteverificationlibrary`, `SGXDataCenterAttestationPrimitives`
- **AMD:** `snpguest` (virtee), `sev-guest` (AMDESE)

**What they do:** Low-level quote verification (signature, measurements).
**What they do NOT do:** Provider/PPID registry lookup.

---

## Summary: Your Positioning

| Project | Collateral Mgmt | Quote Verification | PPID Registry | Provider Lookup |
|---------|-----------------|-------------------|---------------|-----------------|
| Automata DCAP | ✅ | ✅ | ❌ | ❌ |
| Proof of Cloud | ❌ | ✅ | ✅ | ✅ |
| **Your Dashboard** | ❌ | ✅ | ✅ (simple) | ✅ |

**Your value:** A hackathon PoC that demonstrates the full flow (parse → verify → provider lookup) in a way that's open, documented, and easy to extend.

---

# Part 2: How It Should Work

## High-Level Flow

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  User pastes    │     │  Backend         │     │  Provider       │
│  attestation   │────▶│  • Parse quote    │────▶│  Registry       │
│  quote (hex/   │     │  • Verify sig     │     │  (JSON/on-chain)│
│  base64)       │     │  • Extract PPID   │     │                 │
└─────────────────┘     └──────────────────┘     └────────┬────────┘
                                │                         │
                                │                         │
                                ▼                         ▼
┌─────────────────────────────────────────────────────────────────┐
│  Frontend displays:                                              │
│  • TEE valid: Yes/No                                             │
│  • PPID: [extracted value]                                        │
│  • Provider: Trusted (AWS) / Unknown / Not in registry           │
│  • Trust score or status badge                                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Step-by-Step Logic

### Step 1: Input
- User provides an attestation quote (hex-encoded or base64)
- Optionally: User provides quote format (Intel DCAP vs AMD SEV-SNP)

### Step 2: Parse
- Decode the quote structure
- **Intel DCAP:** Quote v3/v4/v5 format; extract `report_data` (first 16 bytes = Platform ID / PPID-derived)
- **AMD SEV-SNP:** Extract chip ID / platform info from report structure

### Step 3: Verify
- Validate attestation signature
- Check certificate chain (or use Intel/AMD verification libraries)
- Verify measurements (MRENCLAVE, MRSIGNER, etc.) if expected values provided

### Step 4: Registry Lookup
- Take extracted PPID / Platform ID
- Look up in provider registry:
  - **Match found** → "Trusted provider: [Provider Name]"
  - **No match** → "Unknown provider" or "Not in registry"

### Step 5: Output
- Display results in clear UI
- Optional: Export as JSON for API consumers

---

## Data Structures

### Provider Registry (Simple JSON)

```json
{
  "version": "1.0",
  "entries": [
    {
      "ppid_hash": "sha256(ppid_or_platform_id)",
      "provider": "AWS",
      "region": "us-east-1",
      "verification_level": 1,
      "added_at": "2025-01-15"
    },
    {
      "ppid_hash": "sha256(...)",
      "provider": "Azure",
      "region": "europe-west",
      "verification_level": 1,
      "added_at": "2025-01-16"
    }
  ]
}
```

**Note:** PPID may be sensitive; registry often stores hashes or salted hashes (as Proof of Cloud does).

### Verification Result

```json
{
  "valid": true,
  "tee_type": "Intel TDX",
  "ppid_extracted": true,
  "provider_match": {
    "found": true,
    "provider": "AWS",
    "region": "us-east-1"
  },
  "measurements": { ... },
  "timestamp": "2025-03-10T12:00:00Z"
}
```

---

# Part 3: How It Should Look (UI/UX for Success)

## Minimal Viable UI

### 1. **Landing / Input Screen**
- Large text area or file upload for attestation quote
- Format selector: Intel DCAP | AMD SEV-SNP | Auto-detect
- "Verify" button

### 2. **Results Screen**
- **Status badge:** Green "Trusted" / Yellow "Unknown" / Red "Invalid"
- **TEE validity:** "Attestation signature valid" ✓
- **PPID / Platform ID:** (masked or truncated for privacy)
- **Provider:** "AWS (us-east-1)" or "Not in registry"
- **Collapsible details:** Raw measurements, certificate info

### 3. **Registry Management (Optional)**
- If you include admin: Add/remove provider entries
- For hackathon: Pre-populate with 3–5 mock entries

---

## Visual Mockup (ASCII)

```
┌────────────────────────────────────────────────────────────────┐
│  PPID Verification Dashboard                                   │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  Paste attestation quote (hex or base64):                       │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │ 0x00000000000000000000000000000000...                     │ │
│  │                                                            │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                                │
│  Format: [Intel DCAP ▼]    [Verify]                             │
│                                                                │
├────────────────────────────────────────────────────────────────┤
│  Results                                                        │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │  Status:  [✓ Trusted Provider]                            │ │
│  │  TEE:     Intel TDX                                       │ │
│  │  PPID:    a1b2c3... (truncated)                           │ │
│  │  Provider: AWS (us-east-1)                                 │ │
│  │  Verification: Signature valid, Provider in registry       │ │
│  └──────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────┘
```

---

## Success Criteria for Judges

1. **Functional:** Paste a (real or mock) attestation → get correct trusted/unknown result
2. **Clear:** Non-experts understand what "Trusted" vs "Unknown" means
3. **Paper-aligned:** Explicitly shows the PPID → provider binding the paper proposes
4. **Extensible:** Registry format allows adding real providers later

---

# Part 4: Stressful / Challenging Areas

## 🔴 High Stress

### 1. **Attestation Format Complexity**
- **Intel DCAP:** Multiple quote versions (v3, v4, v5); binary format; need to parse nested structures
- **AMD SEV-SNP:** Different structure; Chip ID vs PPID semantics
- **Mitigation:** Start with ONE format (Intel DCAP). Use existing libraries (`sgx-dcap-ql`, Cosmian's `intel-sgx-ra`). Use mock/sample quotes for demo.

### 2. **Collateral / Certificate Chain**
- Full verification requires fetching PCK certs, TCB info, CRLs from Intel PCS
- Can fail silently if collateral is expired
- **Mitigation:** For hackathon, you can do *simplified* verification (parse + extract PPID) and note "full chain verification requires collateral" in docs. Or integrate with Automata's collateral service.

### 3. **PPID Extraction Semantics**
- Intel: Platform ID = first 16B of user_data in quote; may be encrypted PPID
- AMD: Chip ID in different field
- **Mitigation:** Rely on Intel/AMD docs and sample quotes. Document assumptions.

---

## 🟡 Medium Stress

### 4. **No Real TEE Hardware**
- You can't generate real attestations without TDX/SEV hardware
- **Mitigation:** Use published sample quotes. Create mock quotes with known PPIDs for demo. Document: "Tested with sample data; production use requires real attestations."

### 5. **Registry Design**
- Real PPIDs are sensitive; hashing/salting adds complexity
- **Mitigation:** For hackathon, use mock PPIDs or hashes. Don't need real provider data.

### 6. **Cross-Platform (Intel + AMD)**
- Two different formats = 2x parsing logic
- **Mitigation:** Ship Intel-only first. AMD as stretch goal.

---

## 🟢 Lower Stress

### 7. **Frontend**
- Standard React/Next.js; straightforward

### 8. **Backend**
- REST API; stateless; no heavy infra

### 9. **Registry Storage**
- JSON file or SQLite is enough

---

# Part 5: Tech Stack Recommendation

| Layer | Recommendation | Rationale |
|-------|----------------|-----------|
| Backend | Rust | You like Rust; good for parsing binary attestation formats |
| Attestation libs | `sgx-dcap-ql` or Cosmian `intel-sgx-ra` (Python) | Existing verification logic |
| Frontend | React + Tailwind | Fast to build; good defaults |
| Registry | JSON file | Simple; easy to swap for DB/on-chain later |
| Deployment | Docker / Vercel + Fly.io | Easy demo deployment |

---

# Part 6: Hackathon Build Plan (5–7 Days)

| Day | Focus |
|-----|-------|
| 1 | Set up project; parse Intel DCAP quote structure; extract Platform ID from sample quote |
| 2 | Implement basic verification (or mock it); create JSON registry with 3–5 mock entries |
| 3 | Backend API: POST /verify; return structured result |
| 4 | Frontend: Input form + results display |
| 5 | Polish UI; add "Unknown provider" vs "Trusted" logic |
| 6 | Documentation; demo script; stretch: AMD support or Proof of Cloud integration |
| 7 | Buffer; practice pitch |

---

# Part 7: References

- **Paper:** Rezabek et al., "Narrowing the Gap between TEEs Threat Model and Deployment Strategies" (arXiv:2506.14964)
- **Proof of Cloud:** https://proofofcloud.org/
- **Automata DCAP:** https://docs.ata.network/dcap-dashboard
- **Intel DCAP:** https://github.com/intel/SGXDataCenterAttestationPrimitives
- **LooseSEAL (Flashbots):** https://collective.flashbots.net/t/loose-seal-enabling-crash-tolerant-tdx-applications/4243

---

*Document version: 1.0 — March 2025*
