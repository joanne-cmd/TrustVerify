# Demo Script — TrustVerify

Use this for hackathon demo. Practice until you can run it in under 2 minutes.

---

## Pre-Demo Setup (Do Before Judges Arrive)

1. Start backend: `cd backend && cargo run`
2. Start frontend: `cd frontend && npm run dev`
3. Open http://localhost:3000
4. Have these ready in separate tabs or notepad:
   - `samples/trusted.hex` (full content)
   - `samples/unknown.hex` (full content)

---

## Demo Flow (≈90 seconds)

### 1. Intro (15 sec)
> "This tool answers: Is this confidential workload running on trusted hardware? It parses TEE attestations, extracts the hardware ID (PPID), and checks it against a provider registry — the gap our paper identifies."

### 2. Trusted Quote (30 sec)
1. Paste contents of `trusted.hex` into the text area
2. Click **Verify**
3. Point out: **Green badge — Trusted**, provider name (e.g., "Demo AWS")
4. Say: "The PPID matches our registry — this workload runs on verified hardware."

### 3. Unknown Quote (25 sec)
1. Clear the text area
2. Paste contents of `unknown.hex`
3. Click **Verify**
4. Point out: **Yellow badge — Unknown**, "Not in registry"
5. Say: "Same valid TEE format, but we don't know who operates this hardware — user can't assess physical attack risk."

### 4. Invalid Quote (15 sec)
1. Paste `deadbeef` or random garbage
2. Click **Verify**
3. Point out: **Red badge — Invalid** or error message
4. Say: "Malformed attestations are rejected."

### 5. CLI (Optional, 15 sec)
```bash
ppid-verify --quote samples/trusted.hex --registry registry.json
```
> "Same verification via CLI for CI/CD pipelines."

### 6. Close (5 sec)
> "We built this open-source so developers can self-host and integrate. Proof of Cloud does something similar at scale; we focus on developer experience and local testing."

---

## Backup If Something Breaks

- **Backend won't start:** Show the code structure, explain the flow
- **Frontend won't start:** Demo CLI only
- **No sample quotes:** Use mock data; explain you'd use real Intel DCAP quotes in production

---

## Talking Points for Q&A

- **"How is this different from Proof of Cloud?"** — Open source, self-hostable, CLI/API for CI/CD, mock registry for local dev.
- **"What's PPID?"** — Protected Platform Identifier; unique per CPU; links attestation to physical hardware.
- **"Why does provider matter?"** — TEEs don't protect against physical attacks; you must trust the facility. Registry binds hardware to verified locations.

---

*Demo Script v1.0*
