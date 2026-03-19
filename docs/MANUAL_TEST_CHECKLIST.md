# Manual Test Checklist

Use this to verify the whole project works. Check each box as you complete it.

---

## Prerequisites
- [ ] Backend starts: `cd backend && cargo run` (Rust)

- [ ] Frontend starts: `cd frontend && npm run dev` (no errors)

---

## Web UI Tests

### Trusted Quote (in registry)
- [ ] Open http://localhost:3000
- [ ] Paste `samples/trusted.hex` into text area
- [ ] Click **Verify**
- [ ] **Status: Trusted** (green badge)
- [ ] Provider name shown (e.g., "Demo AWS")
- [ ] No console errors

### Unknown Quote (not in registry)
- [ ] Clear text area
- [ ] Paste `samples/unknown.hex`
- [ ] Click **Verify**
- [ ] **Status: Unknown** (yellow badge)
- [ ] "Not in registry" or similar message
- [ ] No console errors

### Invalid Quote (malformed)
- [ ] Paste `deadbeef` or random text
- [ ] Click **Verify**
- [ ] **Status: Invalid** (red badge) or error message
- [ ] No crash

---

## CLI Tests

### Trusted
- [ ] `ppid-verify --quote samples/trusted.hex --registry registry.json`
- [ ] Exit code 0
- [ ] Output contains provider name or "Trusted"

### Unknown
- [ ] `ppid-verify --quote samples/unknown.hex --registry registry.json`
- [ ] Exit code 1 (or non-zero)
- [ ] Output contains "Unknown" or "Not in registry"

### Invalid
- [ ] `ppid-verify --quote samples/invalid.hex --registry registry.json`
- [ ] Exit code 2 (or non-zero)
- [ ] Error message shown

---

## API Tests (curl)

### Trusted
```bash
curl -X POST http://localhost:8080/api/verify \
  -H "Content-Type: application/json" \
  -d '{"quote":"<paste trusted.hex>","format":"intel_dcap"}'
```
- [ ] Returns 200
- [ ] JSON has `provider_match.found: true` or `valid: true`

### Unknown
```bash
curl -X POST http://localhost:8080/api/verify \
  -H "Content-Type: application/json" \
  -d '{"quote":"<paste unknown.hex>","format":"intel_dcap"}'
```
- [ ] Returns 200
- [ ] JSON has `provider_match.found: false`

### Invalid
```bash
curl -X POST http://localhost:8080/api/verify \
  -H "Content-Type: application/json" \
  -d '{"quote":"garbage","format":"intel_dcap"}'
```
- [ ] Returns 400 or 200 with `valid: false`

---

## Unit Tests
- [ ] `cargo test` or `pytest` — all pass

---

## Integration Test (if script exists)
- [ ] Backend running
- [ ] `chmod +x scripts/integration_test.sh && ./scripts/integration_test.sh`
- [ ] All tests pass

---

## Sign-Off
- [ ] All boxes above checked
- [ ] Demo script practiced
- [ ] Ready for hackathon

---

*Last run: _______________*
