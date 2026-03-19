# TrustVerify — 5-Day Build Plan

A day-by-day guide with success criteria and testing strategy.

---

# Day 1: Foundation & Quote Parsing

## Goals
- Project structure in place
- Sample attestation data available
- Parse Intel DCAP quote structure and extract Platform ID

## Tasks

### 1.1 Project Setup (1–2 hours)
```
Create:
├── backend/          # Rust
├── frontend/         # Next.js
├── registry.json     # Mock registry
├── samples/          # Sample quotes
└── tests/
```

**Success looks like:**
- [ ] `cargo build` works
- [ ] `npm run dev` starts frontend
- [ ] No errors in console

### 1.2 Get/Create Sample Quotes (1–2 hours)
- Add a mock quote (hex string with known structure) for testing
- Option: Use sample from Intel DCAP docs or Cosmian `intel-sgx-ra` fixtures

**Success looks like:**
- [ ] `samples/mock_intel_quote.hex` exists
- [ ] You know the expected Platform ID / PPID value inside it (for assertion)

### 1.3 Parse Quote & Extract Platform ID (2–3 hours)
- Implement parser for Intel DCAP quote format
- Extract `report_data` or Platform ID from correct offset
- Reference: Intel TDX DCAP API PDF, or use Cosmian lib

**Success looks like:**
- [ ] `parse_quote(hex_string)` returns struct with `platform_id` field
- [ ] For mock quote, extracted value matches expected
- [ ] Unit test: `assert_eq!(parse_quote(MOCK_QUOTE).platform_id, EXPECTED_ID)`

---

## Day 1 Success Checklist
- [ ] Project runs (backend + frontend)
- [ ] Parser extracts Platform ID from mock quote
- [ ] At least 1 unit test passes

---

# Day 2: Registry & Verification Logic

## Goals
- Mock registry with 3–5 entries
- Lookup logic: PPID → provider
- Verification result structure

## Tasks

### 2.1 Create Registry Schema & Data (1 hour)
```json
{
  "version": "1.0",
  "entries": [
    {
      "platform_id_hash": "sha256_of_known_id",
      "provider": "Demo AWS",
      "region": "us-east-1",
      "verification_level": 1
    }
  ]
}
```

**Success looks like:**
- [ ] `registry.json` loads without error
- [ ] At least 1 entry matches your mock quote's Platform ID (or hash)

### 2.2 Registry Lookup (1–2 hours)
- Load registry
- Lookup by Platform ID (or hash)
- Return provider info or "unknown"

**Success looks like:**
- [ ] `lookup(known_platform_id)` → `Some(ProviderInfo)`
- [ ] `lookup(unknown_platform_id)` → `None`
- [ ] Unit test for both cases

### 2.3 Verification Engine (2 hours)
- Combine: parse → extract → lookup
- Return structured result: `{ valid, tee_type, provider, ... }`

**Success looks like:**
- [ ] `verify(quote_hex)` returns JSON with `provider_match.found: true` for mock
- [ ] `verify(invalid_quote)` returns `valid: false` or `provider_match.found: false`
- [ ] Integration test: full flow with mock quote

---

## Day 2 Success Checklist
- [ ] Registry lookup works
- [ ] Full verify flow returns correct provider for mock quote
- [ ] Integration test passes

---

# Day 3: Backend API & CLI

## Goals
- REST API: `POST /api/verify`
- CLI: `ppid-verify --quote file.hex`

## Tasks

### 3.1 REST API (2–3 hours)
```
POST /api/verify
Body: { "quote": "0x...", "format": "intel_dcap" }
Response: { "valid", "tee_type", "provider_match", ... }
```

**Success looks like:**
- [ ] `curl -X POST http://localhost:8080/api/verify -d '{"quote":"..."}'` returns 200 + JSON
- [ ] Valid quote → `provider_match.found: true`
- [ ] Invalid/malformed quote → 400 or `valid: false`

### 3.2 CLI (1–2 hours)
```bash
ppid-verify --quote samples/mock_intel_quote.hex --registry registry.json
```

**Success looks like:**
- [ ] Exit code 0 for trusted, 1 for unknown, 2 for invalid
- [ ] Prints provider name or "Unknown provider"
- [ ] `ppid-verify --help` shows usage

### 3.3 Error Handling
- Malformed input → clear error message
- Missing registry → helpful message

**Success looks like:**
- [ ] No unhandled panics/crashes
- [ ] Errors return meaningful messages

---

## Day 3 Success Checklist
- [ ] API accepts quote and returns verification result
- [ ] CLI works with mock quote
- [ ] Error cases handled gracefully

---

# Day 4: Frontend

## Goals
- Input screen: paste quote, select format, verify button
- Results screen: status badge, provider, details
- Call backend API

## Tasks

### 4.1 Input UI (1–2 hours)
- Textarea for quote (hex or base64)
- Format dropdown: Intel DCAP | AMD SEV-SNP
- "Verify" button

**Success looks like:**
- [ ] User can paste quote and click Verify
- [ ] Loading state while request in flight

### 4.2 Results UI (2 hours)
- Status badge: Trusted (green) | Unknown (yellow) | Invalid (red)
- Provider name, region
- Truncated Platform ID
- Collapsible "Details" section

**Success looks like:**
- [ ] Results display correctly after Verify
- [ ] Badge color matches status
- [ ] Layout is readable on mobile (responsive)

### 4.3 Connect to Backend (1 hour)
- `fetch('/api/verify', { method: 'POST', body: JSON.stringify({ quote }) })`
- Handle errors (network, 400, etc.)

**Success looks like:**
- [ ] Frontend gets result from backend
- [ ] Network errors show user-friendly message

---

## Day 4 Success Checklist
- [ ] Full flow: paste quote → Verify → see results
- [ ] All three statuses (Trusted, Unknown, Invalid) display correctly
- [ ] No console errors in normal flow

---

# Day 5: Testing, Polish & Demo Prep

## Goals
- End-to-end tests
- Documentation
- Demo script
- Deployment (optional)

## Tasks

### 5.1 End-to-End Test (1–2 hours)
- Automated test: start backend → call API with mock quote → assert response
- Or: Playwright/Cypress for frontend E2E

**Success looks like:**
- [ ] E2E test passes in CI or locally
- [ ] Test covers: valid quote → Trusted, unknown quote → Unknown

### 5.2 Manual Test Script (30 min)
- Document exact steps for demo
- Prepare 2–3 quotes: one trusted, one unknown, one invalid

**Success looks like:**
- [ ] `DEMO_SCRIPT.md` exists with copy-paste steps
- [ ] You can run demo in < 2 minutes

### 5.3 README & Docs (1 hour)
- Quickstart
- How to run tests
- What each status means

**Success looks like:**
- [ ] New dev can clone and run in < 10 min
- [ ] README links to paper and Proof of Cloud

### 5.4 Polish (1–2 hours)
- Fix UI glitches
- Add "What is PPID?" explainer section
- Ensure error messages are clear

**Success looks like:**
- [ ] No obvious bugs
- [ ] Demo-ready

---

## Day 5 Success Checklist
- [ ] E2E test passes
- [ ] Demo script works
- [ ] README complete
- [ ] Project is demo-ready

---

# Testing Strategy

## 1. Unit Tests

| Component | What to Test | Example |
|-----------|--------------|---------|
| Parser | Extract Platform ID from valid quote | `parse_quote(MOCK) == expected` |
| Parser | Handle malformed input | `parse_quote("garbage")` → error |
| Registry | Lookup known ID | `lookup(known) == Some(provider)` |
| Registry | Lookup unknown ID | `lookup(unknown) == None` |
| Verifier | Full flow with mock | `verify(MOCK_QUOTE).provider_match.found == true` |

**Run:** `cargo test`

---

## 2. Integration Tests

| Test | Steps | Expected |
|-----|-------|----------|
| API valid quote | POST /api/verify with mock quote | 200, `provider_match.found: true` |
| API unknown quote | POST with quote not in registry | 200, `provider_match.found: false` |
| API invalid quote | POST with garbage | 400 or `valid: false` |
| CLI trusted | `ppid-verify mock.hex` | Exit 0, "Trusted" in output |
| CLI unknown | `ppid-verify unknown.hex` | Exit 1, "Unknown" in output |

**Run:** Start backend, run `./scripts/integration_test.sh` or pytest with live server

---

## 3. End-to-End (E2E) Tests

| Test | Steps | Expected |
|-----|-------|----------|
| Happy path | Open UI → paste quote → Verify | Green "Trusted" badge, provider name |
| Unknown path | Paste quote not in registry | Yellow "Unknown" badge |
| Invalid path | Paste garbage | Red "Invalid" badge or error |
| Error handling | Stop backend, click Verify | Error message shown |

**Run:** Playwright, Cypress, or manual checklist

---

## 4. Manual Test Checklist (Final Verification)

See **[MANUAL_TEST_CHECKLIST.md](MANUAL_TEST_CHECKLIST.md)** for the full checklist.

Summary:
```
□ Backend starts without errors
□ Frontend starts without errors
□ Open http://localhost:3000
□ Paste TRUSTED quote (in registry) → Click Verify
  → Status: Trusted (green)
  → Provider name shown
□ Paste UNKNOWN quote (not in registry) → Click Verify
  → Status: Unknown (yellow)
  → "Not in registry" or similar
□ Paste INVALID quote (garbage hex) → Click Verify
  → Status: Invalid (red) or error message
□ CLI: ppid-verify --quote trusted.hex
  → Exit 0, prints provider
□ CLI: ppid-verify --quote unknown.hex
  → Exit 1, prints "Unknown"
□ All unit tests pass
□ Integration test passes (./scripts/integration_test.sh)
```

---

# Test Data Setup

## Create 3 Quote Files

| File | Purpose | How to Create |
|------|---------|---------------|
| `samples/trusted.hex` | In registry → Trusted | Mock quote with Platform ID matching registry entry |
| `samples/unknown.hex` | Not in registry → Unknown | Mock quote with different Platform ID |
| `samples/invalid.hex` | Malformed → Invalid | `"deadbeef"` or random hex |

## Registry Entry for Trusted Quote

Ensure `registry.json` has an entry whose `platform_id_hash` (or `platform_id`) matches the value you extract from `trusted.hex`. For simplicity in demo, you can use plain Platform ID if not sensitive.

---

# Quick Reference: Success by Day

| Day | Must Have | Nice to Have |
|-----|-----------|--------------|
| 1 | Parser extracts Platform ID, 1 unit test | Both Intel + AMD parsing |
| 2 | Registry lookup, full verify flow works | Multiple registry entries |
| 3 | API + CLI work with mock quote | OpenAPI spec |
| 4 | UI: paste → verify → see results | Educational section |
| 5 | E2E test, demo script, README | Deployed to Vercel/Fly |

---

# If You Fall Behind

| Day | Cut | Keep |
|-----|-----|------|
| 1 | AMD support | Intel only |
| 2 | Fancy registry schema | Simple JSON |
| 3 | CLI | API (more critical for UI) |
| 4 | Collapsible details | Core results display |
| 5 | E2E automation | Manual test checklist |

---

*5-Day Build Plan v1.0 — March 2025*
