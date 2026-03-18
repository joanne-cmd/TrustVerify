# Frontend Code Explainer

TrustVerify frontend: a Next.js App Router app that lets users paste attestation quotes (hex), call the backend verification API, and display results with an optional paravisor-build verification layer.

---

## Overview

| Path | Role |
|------|------|
| `src/app/page.tsx` | Single main page: verify UI, sample quotes, results, security badges, tabs (verify / history / docs / api) |
| `src/app/layout.tsx` | Root layout: HTML shell, Geist fonts, metadata |
| `src/app/globals.css` | Tailwind import, CSS variables, body styles |
| `package.json` | Next 16, React 19, Tailwind 4, TypeScript, ESLint |
| `next.config.ts` | Next.js config (default) |
| `tsconfig.json` | TypeScript: ES2017, path alias `@/*` → `./src/*` |

**Tech:** Next.js 16 (App Router), React 19, TypeScript, Tailwind CSS 4, Geist fonts. No separate state library; all state is local React state in the page component.

---

## 1. Entry and layout: `layout.tsx`

- **Role:** Root layout for the app. Wraps all pages in `<html>` and `<body>`.
- **Fonts:** Loads **Geist** (sans) and **Geist Mono** via `next/font/google`; exposes as CSS variables `--font-geist-sans` and `--font-geist-mono`. Body uses `antialiased` and these variables.
- **Metadata:** `title`: "TrustVerify — PPID Verification Dashboard"; `description` for SEO.
- **Children:** Renders `{children}` (the active page, i.e. `page.tsx`).

---

## 2. Global styles: `globals.css`

- **Tailwind:** `@import "tailwindcss"` (Tailwind v4).
- **Theme:** `@theme inline` sets `--color-background`, `--color-foreground`, and font variables from the layout.
- **Root:** `:root` defines `--background` and `--foreground` (light). Dark mode media query overrides to dark values.
- **Body:** Sets background, color, and fallback font. The main page overrides these with inline styles for a dark UI.

---

## 3. Main page: `page.tsx`

The only route is the default page. The file is a **client component** (`"use client"`) so it can use `useState`, `useEffect`, and event handlers.

### 3.1 Types

- **`BackendResult`** — Matches the backend `/api/verify` JSON: `valid`, `tee_type`, `ppid_extracted`, `platform_id_truncated`, `provider_match` (found, optional provider, region, verification_level, **signature_valid**), `status` ("Trusted" | "Unknown" | "Invalid"), optional `error`, `timestamp`, and the research-facing fields: **`tcb_svn`**, **`tcb_regression`**, **`migration_detected`**, **`registry_sig_valid`**, **`trust_score`**. All of these are displayed in the result section (trust meter, security badges, SVN snippet).

- **`VerificationStep`** — One step in the “verification chain”: `id`, `label`, `status` ("pass" | "warn" | "fail"), `detail` (string). Used to render the step list below the trust meter.

### 3.2 Constants and config

- **`PARAVISOR_REGISTRY`** — Client-side map: paravisor build hash (string) → `{ name, version, repo, buildReproducible, verifiedAt, components[] }`. Used only when the user enters a “Paravisor build hash”; the backend does not see this. Example entries: OpenHCL (Microsoft), COCONUT-SVSM (SUSE), “Proprietary Paravisor”. Drives the optional “Paravisor build verification” step and the paravisor result card.

- **`SAMPLE_QUOTES`** — Predefined quote sets for “Load sample”: `trusted_tdx` (AWS-style), `trusted_azure`, `trusted_simple` (short format), `unknown`, `invalid`. Each has `label`, `hex`, and optionally `paravisorHash`. Loading a sample sets the quote textarea and paravisor field and clears the current result.

- **`Icon`** — Inline SVG helpers: `shield`, `cpu`, `code`, `check`, `warn`, `x`, `copy`. Each takes a `className` string and returns a small SVG (e.g. for header, steps, buttons).

- **`STATUS_CONFIG`** — For statuses `trusted` | `partial` | `unknown` | `invalid`: `label`, `color`, `bg`, `border`. Used for the big status card and styling.

- **`STEP_COLORS`** — `pass` / `warn` / `fail` hex colors for the verification steps.

- **`API_URL`** — `process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080"`. Used for **POST /api/verify** and **GET /api/history**.

### 3.3 TrustMeter component

- **Props:** **`score?: number`** (backend `trust_score` 0–100) and **`status: string`** (e.g. `"trusted"`, `"unknown"`).
- **Behavior:** Uses `score` when provided; otherwise fallback by status (trusted 100%, partial 60%, unknown 30%, invalid 0%). Renders a horizontal bar with that percentage; bar color is green (≥80), amber (≥50), or red (&lt;50). Shows “TRUST SCORE” and “X/100” above the bar. Below the bar, a short breakdown line: “+50 valid quote”, “+25 provider”, “+15 no regression”, “+10 other” (matches backend verifier logic).

### 3.4 Main component state

- **`quoteHex`** — Textarea value: raw hex quote (with or without spaces).
- **`paravisorHash`** — Optional paravisor build hash; if set, the UI looks it up in `PARAVISOR_REGISTRY` and adds a paravisor step and card.
- **`result`** — After a successful verify: `{ status, steps, backend, paravisor }` or `null`. `status` is derived (trusted/partial/unknown/invalid). `steps` are built from the backend response plus optional paravisor step. `backend` is the full `BackendResult`. `paravisor` is the registry entry if found.
- **`loading`** — True while the verify request is in flight.
- **`error`** — User-visible error string (e.g. network failure).
- **`copied`** — True briefly after “Export JSON” copy.
- **`activeTab`** — `"verify"` | `"history"` | `"docs"` | `"api"`. Verify has the form and results; history has the PPID input and history table; docs and api are static content.
- **`scanRef`** — `useRef` for a `setTimeout` handle so the verify run can be debounced/cancelled on unmount.
- **`historyPpid`** — PPID (hex) used for the history tab. After a successful verify, this is auto-set from `platform_id_truncated` with `"..."` removed so the user can load history or paste a full PPID.
- **`historyData`** — Result of **GET /api/history**: `{ ppid, records[], regression, migration, error? }`. `records` are past quotes (timestamp, tcb_svn, provider); `regression` and `migration` are set when the backend detects them.
- **`historyLoading`** — True while the history request is in flight.

### 3.5 Verify flow: `handleVerify`

1. If `quoteHex` is empty, return. Set `loading` true, clear `error` and `result`.
2. After an 800 ms timeout (stored in `scanRef`), run:
   - **POST** `{API_URL}/api/verify` with body `{ quote: quoteHex.trim(), format: "intel_dcap" }`.
   - Parse JSON as `BackendResult`.
3. **Build `steps`:**
   - If `backend.valid`: add steps for hex decoding, DCAP header parse, Platform ID extraction, provider registry lookup (pass if `provider_match.found`, else warn). Detail text uses `platform_id_truncated`, `tee_type`, provider/region when found.
   - If `!backend.valid`: single “Hex decoding” step with status fail and `backend.error`.
4. **Paravisor (client-side):** If `paravisorHash` is non-empty and `backend.valid`, look up hash in `PARAVISOR_REGISTRY`. Add one step “Paravisor build verification” (pass if reproducible, else warn). Set `paravisorResult` for the result card.
5. **Derive `status`:**  
   - If `!backend.valid` → `"invalid"`.  
   - If valid and provider found: if paravisor hash given and paravisor found and reproducible → `"trusted"`; else if paravisor given but not reproducible or unknown → `"partial"`; else no paravisor → `"trusted"`.  
   - If valid and provider not found → `"unknown"`.
6. Call `setResult({ status, steps, backend, paravisor: paravisorResult })`. If `backend.platform_id_truncated` is set, call `setHistoryPpid(backend.platform_id_truncated.replace("...", "").trim())` so the history tab can be used without re-typing.
7. On fetch error, set `error` and a result with status `"invalid"` and a single “API request” fail step.
8. In `finally`, set `loading` false. On unmount, `useEffect` cleanup clears the timeout in `scanRef`.

### 3.6 UI structure (verify tab)

- **Header:** Logo (shield icon), “TRUSTVERIFY” title, subtitle “CONFIDENTIAL COMPUTE ATTESTATION”, and tab buttons (verify / history / docs / api).
- **Intro:** Title “Verify TEE Attestation” and short description (DCAP, Platform ID, provider registry, paravisor).
- **Load sample:** Buttons from `SAMPLE_QUOTES`; each calls `loadSample(key)` to set quote and optional paravisor hash.
- **Inputs:**  
  - Textarea “ATTESTATION QUOTE (HEX)” bound to `quoteHex`.  
  - Input “PARAVISOR BUILD HASH” (optional) bound to `paravisorHash`.
- **Error:** If `error` is set, show a red message box.
- **Verify button:** Disabled when quote is empty or `loading`. On click, `handleVerify`. Label switches to “▶ SCANNING ATTESTATION CHAIN...” when loading.
- **Results (when `result`):**
  - **Status card:** Border/background from `STATUS_CONFIG[result.status]`, icon (check / warn / x), label (TRUSTED / PARTIAL / etc.), short explanation, and “Export JSON” button (copies `{ status, backend, paravisor }` to clipboard).
  - **TrustMeter** with **`score={result.backend?.trust_score}`** and `status={result.status}` so the bar uses the backend’s composite trust score when present.
  - **Verification chain:** List of `result.steps`; each row has a colored left border and dot, label, detail, and status text (PASS / WARN / FAIL). Rows use `step-row` and staggered `animationDelay` for a simple fade-in.
  - **Parsed details:** Two cells — TEE Type and Platform ID (from `result.backend`) when valid.
  - **Provider card:** Only if `provider_match.found`; shows provider name and region.
  - **Security badges (when `result.backend?.valid`):** A row of badges: **TCB** (“✓ TCB CURRENT” green or “⚠ TCB REGRESSION” red from `tcb_regression`), **Migration** (“✓ NO MIGRATION” green or “⚠ HOST MIGRATION” amber from `migration_detected`), **Registry** (“✓ REGISTRY SIGNED” green or “— NO SIGNATURE” gray from `registry_sig_valid`), and **TCB SVN** (first 8 chars of `tcb_svn` when present). These make the backend’s research contribution visible in the UI.
  - **Paravisor card:** Only if `result.paravisor`; shows name, version, reproducible build, and verified components list.

### 3.7 Other tabs

- **history:** “Quote history” — PPID (hex) input and “Load history” button. Calls **`fetchHistory()`**, which does **GET /api/history?ppid=...** and stores the response in `historyData`. Shows an error message if the request fails. If the response includes **regression**, a red alert “TCB regression: previous_svn > current_svn” is shown; if **migration**, an amber alert with previous → current PPID (truncated). A **table** lists past quotes: Timestamp, TCB SVN, Provider. Rows where `rec.tcb_svn === historyData.regression?.previous_svn` are red-highlighted. If there are no records, “No records for this PPID.” is shown.
- **docs:** “Implementation Notes” — four sections: DCAP quote structure, Platform ID / PPID, paravisor verification, threat model gap. Static text and styling.
- **api:** “REST API Reference” — documents **POST /api/verify**: request body `{ quote, format }` and example response JSON. **GET /api/registry** is not yet wired in the UI; judges can call it directly.

### 3.8 Helpers

- **`loadSample(key)`** — Sets `quoteHex` and `paravisorHash` from `SAMPLE_QUOTES[key]`, clears `result` and `error`.
- **`fetchHistory()`** — If `historyPpid` is non-empty, sets `historyLoading` true, requests **GET /api/history?ppid=...**, then sets `historyData` to the JSON (or an object with `error` on failure) and `historyLoading` false.
- **`copyResult()`** — Serializes `result.status`, `result.backend`, `result.paravisor` to JSON, copies to clipboard, sets `copied` true then false after 2s.

### 3.9 Styling

- Inline styles dominate (no separate component CSS files). Dark theme: background `#080808`, borders `#111` / `#1f1f1f`, accent green `#00ff9d`, status colors from `STATUS_CONFIG`.
- A `<style>` block in the page imports Google Fonts (JetBrains Mono, Syne), defines keyframes `pulse` and `fadeUp`, scrollbar and focus styles, and classes for `.tab-btn`, `.sample-btn`, `.verify-btn`, `.step-row`.
- Footer: “TRUSTVERIFY — CONFIDENTIAL COMPUTE ATTESTATION TOOL” and link to the TEE threat model paper (arXiv).

---

## 4. Backend integration

- **POST /api/verify** — Body `{ quote: string, format: "intel_dcap" }`. Response is parsed as `BackendResult`. The UI uses all returned fields: `valid`, `tee_type`, `platform_id_truncated`, `provider_match` (including `signature_valid`), `status`, `error`, `timestamp`, and the research-facing fields **`trust_score`** (drives TrustMeter), **`tcb_svn`**, **`tcb_regression`**, **`migration_detected`**, **`registry_sig_valid`** (drives security badges).
- **GET /api/history?ppid=&lt;hex&gt;** — Used by the **history** tab. Response: `{ ppid, records[], regression?, migration? }`. `records` are past attestations (timestamp, tcb_svn, mr_td, provider). Regression/migration are shown as alerts; the row whose TCB SVN equals `regression.previous_svn` is red-highlighted.
- **GET /api/registry** — Implemented on the backend but not called from this frontend; judges can call it from the API tab or directly.

---

## 5. Scripts and run

- **`npm run dev`** — Next dev server (default port 3000).
- **`npm run build`** — Production build.
- **`npm run start`** — Run production server after build.
- **`npm run lint`** — ESLint (config from eslint-config-next).

Backend is expected at `NEXT_PUBLIC_API_URL` or `http://localhost:8080`. Run backend and frontend side by side for full flow.

---

## 6. Quick reference

- **Single page:** All verification and history UI lives in `src/app/page.tsx` (client component).
- **API base:** `process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080"`.
- **APIs used:** **POST /api/verify** (`{ quote, format }`) and **GET /api/history?ppid=** (history tab).
- **Trust score & badges:** Backend `trust_score` drives the TrustMeter bar; `tcb_regression`, `migration_detected`, `registry_sig_valid`, `tcb_svn` drive the security badges in the result section.
- **Paravisor:** Purely client-side lookup in `PARAVISOR_REGISTRY`; not sent to the backend.
- **Tabs:** verify | history | docs | api.
- **Samples:** `SAMPLE_QUOTES` keys: `trusted_tdx`, `trusted_azure`, `trusted_simple`, `unknown`, `invalid`.
