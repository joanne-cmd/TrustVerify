# TrustVerify

**Verify that confidential workloads run on trusted hardware.**

A Rust-based tool that parses TEE attestations, extracts the hardware ID (PPID), and checks it against a provider registry — closing the gap between TEE threat models and real-world deployment.

Based on: *"Narrowing the Gap between TEEs Threat Model and Deployment Strategies"* (Rezabek et al., arXiv:2506.14964)

---

## Simple Description

> **TrustVerify answers: "Is this confidential workload running on trusted, verified hardware?"** It parses attestation quotes, extracts the Platform ID, and checks it against a provider registry so you know who operates the underlying infrastructure.

---

## Tech Stack

- **Backend:** Rust
- **Frontend:** Next.js
- **Registry:** JSON (extensible to on-chain / Proof of Cloud)

---

## Documentation

Guides and planning docs (build, demo script, checklists) are in a **separate folder** next to this repo: `../ppid-verification-docs`. Treat that directory as its own Git project (`git init`, remote, push) when you publish it.

---

## Quick Start

```bash
# 1. Start the backend (from project root)
cd backend && cargo run

# 2. In another terminal, start the frontend
cd frontend && npm run dev

# 3. Open http://localhost:3000 and paste a sample quote

# CLI (optional)
cd backend && cargo run -- verify --quote ../samples/trusted.hex --registry ../registry.json
```
# TrustVerify
