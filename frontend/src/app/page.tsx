"use client";

import { useState, useEffect, useRef } from "react";

// ─── TYPES ───────────────────────────────────────────────────────────────────
type BackendResult = {
  valid: boolean;
  tee_type: string;
  ppid_extracted: boolean;
  platform_id_truncated: string | null;
  provider_match: {
    found: boolean;
    provider?: string | null;
    region?: string | null;
    verification_level?: number | null;
    signature_valid?: boolean;
  };
  status: "Trusted" | "Unknown" | "Invalid";
  error?: string;
  timestamp: string;
  tcb_svn?: string;
  tcb_regression?: boolean;
  migration_detected?: boolean;
  registry_sig_valid?: boolean;
  trust_score?: number;
};

type VerificationStep = {
  id: string;
  label: string;
  status: "pass" | "warn" | "fail";
  detail: string;
};

// ─── PARAVISOR REGISTRY ──────────────────────────────────────────────────────
const PARAVISOR_REGISTRY: Record<
  string,
  {
    name: string;
    version: string;
    repo: string | null;
    buildReproducible: boolean;
    verifiedAt: string | null;
    components: string[];
  }
> = {
  "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2": {
    name: "OpenHCL (Microsoft)",
    version: "v0.3.0",
    repo: "https://github.com/microsoft/openvmm",
    buildReproducible: true,
    verifiedAt: "2025-01-14",
    components: ["vtl2-kernel", "vmbus-driver", "tpm-emulator"],
  },
  "b2c3d4e5f6a7b2c3d4e5f6a7b2c3d4e5f6a7b2c3d4e5f6a7b2c3d4e5f6a7b2c3": {
    name: "COCONUT-SVSM (SUSE)",
    version: "v0.1.0",
    repo: "https://github.com/coconut-svsm/svsm",
    buildReproducible: true,
    verifiedAt: "2025-02-01",
    components: ["vmpl0-kernel", "sev-snp-vtpm", "fs-driver"],
  },
  "c3d4e5f6a7b8c3d4e5f6a7b8c3d4e5f6a7b8c3d4e5f6a7b8c3d4e5f6a7b8c3d4": {
    name: "Proprietary Paravisor",
    version: "unknown",
    repo: null,
    buildReproducible: false,
    verifiedAt: null,
    components: [],
  },
};

// ─── SAMPLE QUOTES (Intel DCAP v3/v4 format) ──────────────────────────────────
const SAMPLE_QUOTES: Record<
  string,
  { label: string; hex: string; paravisorHash?: string }
> = {
  trusted_tdx: {
    label: "✓ Trusted TDX (AWS)",
    hex:
      "0300" +
      "0200" +
      "81000000" +
      "00000000" +
      "939a7233f79c4ca9" +
      "940a0db3957f3465" +
      "aabbccdd11223344aabbccdd11223344" +
      "0000" +
      "00".repeat(200),
    paravisorHash:
      "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
  },
  trusted_azure: {
    label: "✓ Trusted TDX (Azure)",
    hex:
      "0300" +
      "0200" +
      "81000000" +
      "00000000" +
      "939a7233f79c4ca9" +
      "940a0db3957f3465" +
      "deadbeef00112233deadbeef00112233" +
      "0000" +
      "00".repeat(200),
    paravisorHash:
      "b2c3d4e5f6a7b2c3d4e5f6a7b2c3d4e5f6a7b2c3d4e5f6a7b2c3d4e5f6a7b2c3",
  },
  trusted_simple: {
    label: "✓ Trusted (Simple format)",
    hex: "a1b2c3d4e5f6789012345678abcdef0100000000000000000000000000000000000000000000000000000000000000",
  },
  unknown: {
    label: "? Unknown Provider",
    hex:
      "0300" +
      "0200" +
      "00000000" +
      "00000000" +
      "939a7233f79c4ca9" +
      "940a0db3957f3465" +
      "ffffffff000000000000000011111111" +
      "0000" +
      "00".repeat(200),
    paravisorHash:
      "c3d4e5f6a7b8c3d4e5f6a7b8c3d4e5f6a7b8c3d4e5f6a7b8c3d4e5f6a7b8c3d4",
  },
  invalid: {
    label: "✗ Invalid / Malformed",
    hex: "deadbeef",
  },
};

// ─── ICONS ───────────────────────────────────────────────────────────────────
const Icon = {
  shield: (cls: string) => (
    <svg
      className={cls}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 013.598 6 11.955 11.955 0 003 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z"
      />
    </svg>
  ),
  cpu: (cls: string) => (
    <svg
      className={cls}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M8.25 3v1.5M4.5 8.25H3m18 0h-1.5M4.5 12H3m18 0h-1.5m-15 3.75H3m18 0h-1.5M8.25 19.5V21M12 3v1.5m0 15V21m3.75-18v1.5m0 15V21m-9-1.5h10.5a2.25 2.25 0 002.25-2.25V6.75a2.25 2.25 0 00-2.25-2.25H6.75A2.25 2.25 0 004.5 6.75v10.5a2.25 2.25 0 002.25 2.25zm.75-12h9v9h-9v-9z"
      />
    </svg>
  ),
  code: (cls: string) => (
    <svg
      className={cls}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M17.25 6.75L22.5 12l-5.25 5.25m-10.5 0L1.5 12l5.25-5.25m7.5-3l-4.5 16.5"
      />
    </svg>
  ),
  check: (cls: string) => (
    <svg
      className={cls}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M4.5 12.75l6 6 9-13.5"
      />
    </svg>
  ),
  warn: (cls: string) => (
    <svg
      className={cls}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z"
      />
    </svg>
  ),
  x: (cls: string) => (
    <svg
      className={cls}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
    >
      <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
    </svg>
  ),
  copy: (cls: string) => (
    <svg
      className={cls}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M15.75 17.25v3.375c0 .621-.504 1.125-1.125 1.125h-9.75a1.125 1.125 0 01-1.125-1.125V7.875c0-.621.504-1.125 1.125-1.125H6.75a9.06 9.06 0 011.5.124m7.5 10.376h3.375c.621 0 1.125-.504 1.125-1.125V11.25c0-4.46-3.243-8.161-7.5-8.976a9.06 9.06 0 00-1.5-.124H9.375c-.621 0-1.125.504-1.125 1.125v3.5m7.5 10.375H9.375a1.125 1.125 0 01-1.125-1.125v-9.25m12 6.625v-1.875a3.375 3.375 0 00-3.375-3.375h-1.5a1.125 1.125 0 01-1.125-1.125v-1.5a3.375 3.375 0 00-3.375-3.375H9.375"
      />
    </svg>
  ),
};

// ─── STATUS CONFIG ───────────────────────────────────────────────────────────
const STATUS_CONFIG: Record<
  string,
  { label: string; color: string; bg: string; border: string }
> = {
  trusted: {
    label: "TRUSTED",
    color: "#00ff9d",
    bg: "rgba(0,255,157,0.08)",
    border: "rgba(0,255,157,0.3)",
  },
  partial: {
    label: "PARTIAL",
    color: "#ffb800",
    bg: "rgba(255,184,0,0.08)",
    border: "rgba(255,184,0,0.3)",
  },
  unknown: {
    label: "UNKNOWN",
    color: "#ff6b35",
    bg: "rgba(255,107,53,0.08)",
    border: "rgba(255,107,53,0.3)",
  },
  invalid: {
    label: "INVALID",
    color: "#ff3366",
    bg: "rgba(255,51,102,0.08)",
    border: "rgba(255,51,102,0.3)",
  },
};

const STEP_COLORS = { pass: "#00ff9d", warn: "#ffb800", fail: "#ff3366" };

const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080";

// ─── TRUST METER ─────────────────────────────────────────────────────────────
function TrustMeter({ score, status }: { score?: number; status: string }) {
  const fallback: Record<string, number> = {
    trusted: 100,
    partial: 60,
    unknown: 30,
    invalid: 0,
  };
  const pct = score ?? fallback[status] ?? 0;
  const color =
    pct >= 80 ? "#00ff9d" : pct >= 50 ? "#f59e0b" : "#ef4444";
  return (
    <div style={{ margin: "16px 0" }}>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          fontSize: 11,
          color: "#666",
          marginBottom: 6,
        }}
      >
        <span>TRUST SCORE</span>
        <span style={{ color, fontFamily: "var(--font-geist-mono, monospace)" }}>
          {pct}/100
        </span>
      </div>
      <div
        style={{
          background: "#111",
          borderRadius: 4,
          height: 6,
          overflow: "hidden",
        }}
      >
        <div
          style={{
            width: `${pct}%`,
            height: "100%",
            borderRadius: 4,
            background: color,
            transition: "width 0.6s ease",
          }}
        />
      </div>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          fontSize: 10,
          color: "#444",
          marginTop: 4,
        }}
      >
        <span>+50 valid quote</span>
        <span>+25 provider</span>
        <span>+15 no regression</span>
        <span>+10 other</span>
      </div>
    </div>
  );
}

// ─── MAIN COMPONENT ───────────────────────────────────────────────────────────
export default function TrustVerify() {
  const [quoteHex, setQuoteHex] = useState("");
  const [paravisorHash, setParavisorHash] = useState("");
  const [result, setResult] = useState<{
    status: string;
    steps: VerificationStep[];
    backend: BackendResult | null;
    paravisor: (typeof PARAVISOR_REGISTRY)[string] | null;
  } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [activeTab, setActiveTab] = useState("verify");
  const scanRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [historyPpid, setHistoryPpid] = useState("");
  const [historyData, setHistoryData] = useState<{
    ppid: string;
    records: { ppid: string; tcb_svn: string; mr_td: string; timestamp: string; provider: string | null }[];
    regression: { previous_svn: string; current_svn: string } | null;
    migration: { previous_ppid: string; current_ppid: string } | null;
    error?: string;
  } | null>(null);
  const [historyLoading, setHistoryLoading] = useState(false);

  const handleVerify = async () => {
    if (!quoteHex.trim()) return;
    setLoading(true);
    setError(null);
    setResult(null);
    scanRef.current = setTimeout(async () => {
      try {
        const res = await fetch(`${API_URL}/api/verify`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            quote: quoteHex.trim(),
            format: "intel_dcap",
          }),
        });
        const backend: BackendResult = await res.json();

        // Build verification steps
        const steps: VerificationStep[] = [];
        if (backend.valid) {
          steps.push({
            id: "decode",
            label: "Hex decoding",
            status: "pass",
            detail: `${(quoteHex.replace(/\s/g, "").length / 2)} bytes decoded`,
          });
          steps.push({
            id: "header",
            label: "DCAP header parse",
            status: "pass",
            detail: `${backend.tee_type}`,
          });
          steps.push({
            id: "ppid",
            label: "Platform ID extraction",
            status: "pass",
            detail: backend.platform_id_truncated
              ? `0x${backend.platform_id_truncated}`
              : "Extracted",
          });
          steps.push({
            id: "registry",
            label: "Provider registry lookup",
            status: backend.provider_match.found ? "pass" : "warn",
            detail: backend.provider_match.found
              ? `${backend.provider_match.provider} • ${backend.provider_match.region}`
              : "Platform ID not in known registry",
          });
        } else {
          steps.push({
            id: "decode",
            label: "Hex decoding",
            status: "fail",
            detail: backend.error ?? "Parse failed",
          });
        }

        // Paravisor verification (client-side)
        let paravisorResult: (typeof PARAVISOR_REGISTRY)[string] | null = null;
        if (paravisorHash.trim() && backend.valid) {
          const ph = paravisorHash.trim().toLowerCase();
          paravisorResult = PARAVISOR_REGISTRY[ph] ?? null;
          steps.push({
            id: "paravisor",
            label: "Paravisor build verification",
            status: paravisorResult?.buildReproducible
              ? "pass"
              : paravisorResult
                ? "warn"
                : "warn",
            detail: paravisorResult?.buildReproducible
              ? `${paravisorResult.name} ${paravisorResult.version} — reproducible ✓`
              : paravisorResult
                ? `${paravisorResult.name} — NOT reproducible`
                : "Paravisor hash not in verified registry",
          });
        }

        // Overall status
        let status = "invalid";
        if (backend.valid) {
          if (backend.provider_match.found) {
            if (
              paravisorHash.trim() &&
              paravisorResult &&
              paravisorResult.buildReproducible
            ) {
              status = "trusted";
            } else if (
              paravisorHash.trim() &&
              (!paravisorResult || !paravisorResult.buildReproducible)
            ) {
              status = "partial";
            } else {
              status = "trusted";
            }
          } else {
            status = "unknown";
          }
        }

        setResult({
          status,
          steps,
          backend,
          paravisor: paravisorResult,
        });
        if (backend.platform_id_truncated) {
          setHistoryPpid(backend.platform_id_truncated.replace("...", "").trim());
        }
      } catch (err) {
        setError(
          err instanceof Error
            ? err.message
            : "Failed to connect. Is the backend running on port 8080?"
        );
        setResult({
          status: "invalid",
          steps: [
            {
              id: "network",
              label: "API request",
              status: "fail",
              detail: "Could not reach verification service",
            },
          ],
          backend: null,
          paravisor: null,
        });
      } finally {
        setLoading(false);
      }
    }, 800);
  };

  useEffect(
    () => () => {
      if (scanRef.current) clearTimeout(scanRef.current);
    },
    []
  );

  const loadSample = (key: string) => {
    const s = SAMPLE_QUOTES[key];
    if (s) {
      setQuoteHex(s.hex);
      setParavisorHash(s.paravisorHash ?? "");
      setResult(null);
      setError(null);
    }
  };

  const fetchHistory = async () => {
    if (!historyPpid.trim()) return;
    setHistoryLoading(true);
    setHistoryData(null);
    try {
      const res = await fetch(
        `${API_URL}/api/history?ppid=${encodeURIComponent(historyPpid.trim())}`
      );
      const data = await res.json();
      if (!res.ok) throw new Error(data.error || res.statusText);
      setHistoryData(data);
    } catch (err) {
      setHistoryData({
        ppid: historyPpid,
        records: [],
        regression: null,
        migration: null,
        error: err instanceof Error ? err.message : "Failed to load history",
      });
    } finally {
      setHistoryLoading(false);
    }
  };

  const copyResult = () => {
    if (!result) return;
    const out = JSON.stringify(
      {
        status: result.status,
        backend: result.backend,
        paravisor: result.paravisor,
      },
      null,
      2
    );
    navigator.clipboard?.writeText(out);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const cfg = result ? STATUS_CONFIG[result.status] : null;

  return (
    <div
      style={{
        minHeight: "100vh",
        background: "#080808",
        color: "#e0e0e0",
        fontFamily: "'JetBrains Mono', 'Fira Code', 'Courier New', monospace",
        padding: "0 0 60px",
      }}
    >
      <style>{`
        @import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@300;400;500;700&family=Syne:wght@400;700;800&display=swap');
        @keyframes pulse { 0%,100%{opacity:1} 50%{opacity:0.4} }
        @keyframes fadeUp { from{opacity:0;transform:translateY(12px)} to{opacity:1;transform:translateY(0)} }
        ::-webkit-scrollbar{width:4px} ::-webkit-scrollbar-track{background:#111}
        ::-webkit-scrollbar-thumb{background:#333;border-radius:2px}
        textarea:focus,input:focus{outline:none}
        .tab-btn{cursor:pointer;transition:all 0.2s}
        .tab-btn:hover{color:#fff}
        .sample-btn{cursor:pointer;transition:all 0.15s;border:1px solid #1f1f1f;background:#111;color:#666;padding:6px 12px;border-radius:4px;font-family:monospace;font-size:11px}
        .sample-btn:hover{border-color:#333;color:#aaa;background:#161616}
        .verify-btn{cursor:pointer;transition:all 0.2s;border:none;font-family:monospace}
        .verify-btn:hover:not(:disabled){filter:brightness(1.1);transform:translateY(-1px)}
        .verify-btn:disabled{opacity:0.5;cursor:not-allowed}
        .step-row{animation:fadeUp 0.3s ease forwards;opacity:0}
      `}</style>

      {/* Header */}
      <div
        style={{
          borderBottom: "1px solid #111",
          padding: "24px 40px",
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 14 }}>
          <div
            style={{
              width: 32,
              height: 32,
              border: "1.5px solid #00ff9d",
              borderRadius: 6,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            {Icon.shield("w-4 h-4")}
          </div>
          <div>
            <div
              style={{
                fontFamily: "'Syne', sans-serif",
                fontWeight: 800,
                fontSize: 18,
                letterSpacing: "0.05em",
                color: "#fff",
              }}
            >
              TRUST<span style={{ color: "#00ff9d" }}>VERIFY</span>
            </div>
            <div
              style={{
                fontSize: 10,
                color: "#444",
                letterSpacing: "0.15em",
                marginTop: 1,
              }}
            >
              CONFIDENTIAL COMPUTE ATTESTATION
            </div>
          </div>
        </div>
        <div style={{ display: "flex", gap: 24 }}>
          {["verify", "history", "docs", "api"].map((t) => (
            <button
              key={t}
              className="tab-btn"
              onClick={() => setActiveTab(t)}
              style={{
                background: "none",
                border: "none",
                cursor: "pointer",
                fontSize: 11,
                letterSpacing: "0.12em",
                textTransform: "uppercase",
                color: activeTab === t ? "#00ff9d" : "#444",
                borderBottom:
                  activeTab === t ? "1px solid #00ff9d" : "1px solid transparent",
                paddingBottom: 2,
                transition: "all 0.2s",
              }}
            >
              {t}
            </button>
          ))}
        </div>
      </div>

      {activeTab === "verify" && (
        <div
          style={{ maxWidth: 900, margin: "0 auto", padding: "40px 40px 0" }}
        >
          {/* Intro */}
          <div style={{ marginBottom: 36 }}>
            <h1
              style={{
                fontFamily: "'Syne', sans-serif",
                fontWeight: 700,
                fontSize: 28,
                color: "#fff",
                margin: "0 0 10px",
                letterSpacing: "-0.02em",
              }}
            >
              Verify TEE Attestation
            </h1>
            <p
              style={{
                color: "#555",
                fontSize: 13,
                margin: 0,
                lineHeight: 1.7,
              }}
            >
              Parses Intel DCAP quote headers (v3/v4), extracts the Platform ID,
              checks it against a provider registry, and verifies paravisor build
              reproducibility.
            </p>
          </div>

          {/* Sample buttons */}
          <div style={{ marginBottom: 20 }}>
            <div
              style={{
                fontSize: 10,
                color: "#444",
                letterSpacing: "0.12em",
                marginBottom: 8,
              }}
            >
              LOAD SAMPLE
            </div>
            <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
              {Object.entries(SAMPLE_QUOTES).map(([k, v]) => (
                <button
                  key={k}
                  className="sample-btn"
                  onClick={() => loadSample(k)}
                >
                  {v.label}
                </button>
              ))}
            </div>
          </div>

          {/* Inputs */}
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "1fr 1fr",
              gap: 16,
              marginBottom: 16,
            }}
          >
            <div style={{ gridColumn: "1 / -1" }}>
              <label
                style={{
                  display: "block",
                  fontSize: 10,
                  color: "#555",
                  letterSpacing: "0.12em",
                  marginBottom: 8,
                }}
              >
                ATTESTATION QUOTE (HEX)
                <span style={{ color: "#333", marginLeft: 8 }}>
                  Intel DCAP v3/v4 — SGX or TDX
                </span>
              </label>
              <textarea
                value={quoteHex}
                onChange={(e) => {
                  setQuoteHex(e.target.value);
                  setResult(null);
                }}
                placeholder="0300 0200 81000000 00000000 939a7233..."
                rows={5}
                style={{
                  width: "100%",
                  boxSizing: "border-box",
                  background: "#0d0d0d",
                  border: `1px solid ${loading ? "#00ff9d44" : "#1f1f1f"}`,
                  borderRadius: 6,
                  padding: "14px 16px",
                  color: "#b0ffc8",
                  fontSize: 12,
                  lineHeight: 1.6,
                  resize: "vertical",
                  fontFamily: "monospace",
                  transition: "border-color 0.3s",
                }}
              />
            </div>
            <div style={{ gridColumn: "1 / -1" }}>
              <label
                style={{
                  display: "block",
                  fontSize: 10,
                  color: "#555",
                  letterSpacing: "0.12em",
                  marginBottom: 8,
                }}
              >
                PARAVISOR BUILD HASH
                <span style={{ color: "#333", marginLeft: 8 }}>
                  SHA-256 of paravisor image (optional)
                </span>
              </label>
              <input
                value={paravisorHash}
                onChange={(e) => {
                  setParavisorHash(e.target.value);
                  setResult(null);
                }}
                placeholder="a1b2c3d4e5f6... (optional — enables paravirtualization flow verification)"
                style={{
                  width: "100%",
                  boxSizing: "border-box",
                  background: "#0d0d0d",
                  border: "1px solid #1f1f1f",
                  borderRadius: 6,
                  padding: "12px 16px",
                  color: "#b0c8ff",
                  fontSize: 12,
                  fontFamily: "monospace",
                }}
              />
            </div>
          </div>

          {error && (
            <div
              style={{
                marginBottom: 16,
                padding: "12px 16px",
                background: "rgba(255,51,102,0.1)",
                border: "1px solid rgba(255,51,102,0.3)",
                borderRadius: 6,
                color: "#ff3366",
                fontSize: 12,
              }}
            >
              {error}
            </div>
          )}

          {/* Verify button */}
          <button
            className="verify-btn"
            onClick={handleVerify}
            disabled={!quoteHex.trim() || loading}
            style={{
              width: "100%",
              padding: "14px",
              borderRadius: 6,
              background: loading ? "#0d1f14" : "#00ff9d",
              color: loading ? "#00ff9d" : "#000",
              fontSize: 12,
              fontWeight: 700,
              letterSpacing: "0.15em",
              marginBottom: 32,
            }}
          >
            {loading ? (
              <span style={{ animation: "pulse 0.8s ease infinite" }}>
                ▶ SCANNING ATTESTATION CHAIN...
              </span>
            ) : (
              "▶ VERIFY ATTESTATION"
            )}
          </button>

          {/* Results */}
          {result && (
            <div style={{ animation: "fadeUp 0.4s ease" }}>
              <div
                style={{
                  border: `1px solid ${cfg?.border ?? "#333"}`,
                  borderRadius: 8,
                  background: cfg?.bg ?? "#111",
                  padding: "20px 24px",
                  marginBottom: 24,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                }}
              >
                <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
                  <div
                    style={{
                      width: 48,
                      height: 48,
                      borderRadius: 8,
                      border: `1.5px solid ${cfg?.color ?? "#333"}`,
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "center",
                      color: cfg?.color ?? "#333",
                      boxShadow: `0 0 16px ${cfg?.color ?? "#333"}44`,
                    }}
                  >
                    {result.status === "trusted"
                      ? Icon.check("w-6 h-6")
                      : result.status === "invalid"
                        ? Icon.x("w-6 h-6")
                        : Icon.warn("w-6 h-6")}
                  </div>
                  <div>
                    <div
                      style={{
                        fontSize: 22,
                        fontWeight: 700,
                        color: cfg?.color ?? "#333",
                        letterSpacing: "0.05em",
                        fontFamily: "'Syne', sans-serif",
                      }}
                    >
                      {cfg?.label ?? result.status.toUpperCase()}
                    </div>
                    <div style={{ fontSize: 12, color: "#666", marginTop: 2 }}>
                      {result.status === "trusted" &&
                        "Platform verified — known provider"}
                      {result.status === "partial" &&
                        "Provider verified — paravisor build not reproducible"}
                      {result.status === "unknown" &&
                        "Platform ID not in registry — cannot confirm provider"}
                      {result.status === "invalid" &&
                        "Quote could not be parsed — check format"}
                    </div>
                  </div>
                </div>
                <button
                  onClick={copyResult}
                  style={{
                    background: "none",
                    border: "1px solid #1f1f1f",
                    borderRadius: 4,
                    color: "#555",
                    padding: "8px 12px",
                    cursor: "pointer",
                    display: "flex",
                    alignItems: "center",
                    gap: 6,
                    fontSize: 11,
                    fontFamily: "monospace",
                    transition: "all 0.15s",
                  }}
                >
                  {Icon.copy("w-3 h-3")}
                  {copied ? "COPIED" : "EXPORT JSON"}
                </button>
              </div>

              <TrustMeter score={result.backend?.trust_score} status={result.status} />

              {/* Verification steps */}
              <div style={{ marginBottom: 24 }}>
                <div
                  style={{
                    fontSize: 10,
                    color: "#444",
                    letterSpacing: "0.12em",
                    marginBottom: 12,
                  }}
                >
                  VERIFICATION CHAIN
                </div>
                {result.steps.map((step, i) => (
                  <div
                    key={step.id}
                    className="step-row"
                    style={{
                      animationDelay: `${i * 80}ms`,
                      display: "flex",
                      alignItems: "center",
                      gap: 14,
                      padding: "10px 14px",
                      marginBottom: 4,
                      background: "#0d0d0d",
                      borderRadius: 4,
                      borderLeft: `2px solid ${STEP_COLORS[step.status]}`,
                    }}
                  >
                    <div
                      style={{
                        width: 6,
                        height: 6,
                        borderRadius: "50%",
                        background: STEP_COLORS[step.status],
                        flexShrink: 0,
                        boxShadow: `0 0 6px ${STEP_COLORS[step.status]}`,
                      }}
                    />
                    <div
                      style={{
                        fontSize: 12,
                        color: "#888",
                        flex: "0 0 220px",
                      }}
                    >
                      {step.label}
                    </div>
                    <div
                      style={{
                        fontSize: 12,
                        color:
                          STEP_COLORS[step.status] === "#ff3366"
                            ? "#ff3366"
                            : "#ccc",
                      }}
                    >
                      {step.detail}
                    </div>
                    <div
                      style={{
                        marginLeft: "auto",
                        fontSize: 10,
                        color: STEP_COLORS[step.status],
                        letterSpacing: "0.1em",
                      }}
                    >
                      {step.status.toUpperCase()}
                    </div>
                  </div>
                ))}
              </div>

              {/* Parsed details */}
              {result.backend?.valid && result.backend.platform_id_truncated && (
                <div
                  style={{
                    display: "grid",
                    gridTemplateColumns: "1fr 1fr",
                    gap: 12,
                    marginBottom: 24,
                  }}
                >
                  {[
                    ["TEE Type", result.backend.tee_type],
                    ["Platform ID", result.backend.platform_id_truncated],
                  ].map(([k, v]) => (
                    <div
                      key={k}
                      style={{
                        background: "#0d0d0d",
                        borderRadius: 4,
                        padding: "12px 14px",
                        border: "1px solid #141414",
                      }}
                    >
                      <div
                        style={{
                          fontSize: 10,
                          color: "#444",
                          letterSpacing: "0.1em",
                          marginBottom: 4,
                        }}
                      >
                        {k}
                      </div>
                      <div
                        style={{
                          fontSize: 12,
                          color: "#ccc",
                          wordBreak: "break-all",
                        }}
                      >
                        {v}
                      </div>
                    </div>
                  ))}
                </div>
              )}

              {/* Provider card */}
              {result.backend?.provider_match.found && (
                <div
                  style={{
                    background: "#0d0d0d",
                    border: "1px solid #141414",
                    borderRadius: 6,
                    padding: "16px 20px",
                    marginBottom: 16,
                  }}
                >
                  <div
                    style={{
                      fontSize: 10,
                      color: "#444",
                      letterSpacing: "0.12em",
                      marginBottom: 12,
                    }}
                  >
                    {Icon.cpu("w-3 h-3 inline mr-2")}PROVIDER
                  </div>
                  <div
                    style={{
                      display: "flex",
                      gap: 24,
                      flexWrap: "wrap",
                    }}
                  >
                    <div>
                      <div
                        style={{
                          fontSize: 10,
                          color: "#555",
                          marginBottom: 2,
                        }}
                      >
                        NAME
                      </div>
                      <div
                        style={{
                          fontSize: 13,
                          color: "#00ff9d",
                        }}
                      >
                        {result.backend.provider_match.provider}
                      </div>
                    </div>
                    <div>
                      <div
                        style={{
                          fontSize: 10,
                          color: "#555",
                          marginBottom: 2,
                        }}
                      >
                        REGION
                      </div>
                      <div style={{ fontSize: 13, color: "#ccc" }}>
                        {result.backend.provider_match.region}
                      </div>
                    </div>
                  </div>
                </div>
              )}

              {/* Security signals: TCB regression, migration, registry sig */}
              {result.backend?.valid && (
                <div
                  style={{
                    display: "flex",
                    gap: 8,
                    flexWrap: "wrap",
                    margin: "12px 0",
                    marginBottom: 16,
                  }}
                >
                  <div
                    style={{
                      padding: "6px 12px",
                      borderRadius: 4,
                      fontSize: 11,
                      fontFamily: "var(--font-geist-mono, monospace)",
                      background: result.backend.tcb_regression ? "#2d0a0a" : "#0a1f0a",
                      border: `1px solid ${result.backend.tcb_regression ? "#ef4444" : "#00ff9d"}`,
                      color: result.backend.tcb_regression ? "#ef4444" : "#00ff9d",
                    }}
                  >
                    {result.backend.tcb_regression ? "⚠ TCB REGRESSION" : "✓ TCB CURRENT"}
                  </div>
                  <div
                    style={{
                      padding: "6px 12px",
                      borderRadius: 4,
                      fontSize: 11,
                      fontFamily: "var(--font-geist-mono, monospace)",
                      background: result.backend.migration_detected ? "#2d1a0a" : "#0a1f0a",
                      border: `1px solid ${result.backend.migration_detected ? "#f59e0b" : "#00ff9d"}`,
                      color: result.backend.migration_detected ? "#f59e0b" : "#00ff9d",
                    }}
                  >
                    {result.backend.migration_detected ? "⚠ HOST MIGRATION" : "✓ NO MIGRATION"}
                  </div>
                  <div
                    style={{
                      padding: "6px 12px",
                      borderRadius: 4,
                      fontSize: 11,
                      fontFamily: "var(--font-geist-mono, monospace)",
                      background: result.backend.registry_sig_valid ? "#0a1f0a" : "#1a1a0a",
                      border: `1px solid ${result.backend.registry_sig_valid ? "#00ff9d" : "#666"}`,
                      color: result.backend.registry_sig_valid ? "#00ff9d" : "#666",
                    }}
                  >
                    {result.backend.registry_sig_valid ? "✓ REGISTRY SIGNED" : "— NO SIGNATURE"}
                  </div>
                  {result.backend.tcb_svn && (
                    <div
                      style={{
                        padding: "6px 12px",
                        borderRadius: 4,
                        fontSize: 11,
                        fontFamily: "var(--font-geist-mono, monospace)",
                        background: "#0a0a1f",
                        border: "1px solid #333",
                        color: "#888",
                      }}
                    >
                      SVN: {result.backend.tcb_svn.slice(0, 8)}...
                    </div>
                  )}
                </div>
              )}

              {/* Paravisor card */}
              {result.paravisor && (
                <div
                  style={{
                    background: "#0d0d0d",
                    border: "1px solid #141414",
                    borderRadius: 6,
                    padding: "16px 20px",
                  }}
                >
                  <div
                    style={{
                      fontSize: 10,
                      color: "#444",
                      letterSpacing: "0.12em",
                      marginBottom: 12,
                    }}
                  >
                    {Icon.code("w-3 h-3 inline mr-2")}PARAVISOR VERIFICATION
                  </div>
                  <div
                    style={{
                      display: "flex",
                      gap: 24,
                      flexWrap: "wrap",
                      marginBottom: 12,
                    }}
                  >
                    <div>
                      <div
                        style={{
                          fontSize: 10,
                          color: "#555",
                          marginBottom: 2,
                        }}
                      >
                        NAME
                      </div>
                      <div
                        style={{
                          fontSize: 13,
                          color: "#b0c8ff",
                        }}
                      >
                        {result.paravisor.name}
                      </div>
                    </div>
                    <div>
                      <div
                        style={{
                          fontSize: 10,
                          color: "#555",
                          marginBottom: 2,
                        }}
                      >
                        VERSION
                      </div>
                      <div style={{ fontSize: 13, color: "#ccc" }}>
                        {result.paravisor.version}
                      </div>
                    </div>
                    <div>
                      <div
                        style={{
                          fontSize: 10,
                          color: "#555",
                          marginBottom: 2,
                        }}
                      >
                        REPRODUCIBLE BUILD
                      </div>
                      <div
                        style={{
                          fontSize: 13,
                          color: result.paravisor.buildReproducible
                            ? "#00ff9d"
                            : "#ff6b35",
                        }}
                      >
                        {result.paravisor.buildReproducible ? "✓ YES" : "✗ NO"}
                      </div>
                    </div>
                  </div>
                  {result.paravisor.components.length > 0 && (
                    <div>
                      <div
                        style={{
                          fontSize: 10,
                          color: "#555",
                          marginBottom: 6,
                        }}
                      >
                        VERIFIED COMPONENTS
                      </div>
                      <div
                        style={{
                          display: "flex",
                          gap: 8,
                          flexWrap: "wrap",
                        }}
                      >
                        {result.paravisor.components.map((c) => (
                          <span
                            key={c}
                            style={{
                              fontSize: 11,
                              color: "#666",
                              background: "#141414",
                              padding: "3px 10px",
                              borderRadius: 3,
                              border: "1px solid #1f1f1f",
                            }}
                          >
                            {c}
                          </span>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {activeTab === "history" && (
        <div style={{ maxWidth: 900, margin: "0 auto", padding: "40px 40px 0" }}>
          <h1
            style={{
              fontFamily: "'Syne', sans-serif",
              fontWeight: 700,
              fontSize: 28,
              color: "#fff",
              margin: "0 0 10px",
              letterSpacing: "-0.02em",
            }}
          >
            Quote history
          </h1>
          <p
            style={{
              color: "#555",
              fontSize: 13,
              margin: "0 0 24px",
              lineHeight: 1.7,
            }}
          >
            Past attestations for a PPID. Use the full Platform ID (hex). Regression rows are highlighted.
          </p>
          <div style={{ marginBottom: 16 }}>
            <label
              style={{
                display: "block",
                fontSize: 10,
                color: "#555",
                letterSpacing: "0.12em",
                marginBottom: 8,
              }}
            >
              PPID (HEX)
            </label>
            <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
              <input
                type="text"
                value={historyPpid}
                onChange={(e) => setHistoryPpid(e.target.value)}
                placeholder="e.g. aabbccdd11223344aabbccdd11223344"
                style={{
                  flex: 1,
                  maxWidth: 480,
                  background: "#0d0d0d",
                  border: "1px solid #1f1f1f",
                  borderRadius: 6,
                  padding: "12px 16px",
                  color: "#b0ffc8",
                  fontSize: 12,
                  fontFamily: "monospace",
                }}
              />
              <button
                onClick={fetchHistory}
                disabled={!historyPpid.trim() || historyLoading}
                className="verify-btn"
                style={{
                  padding: "12px 20px",
                  borderRadius: 6,
                  background: historyLoading ? "#0d1f14" : "#00ff9d",
                  color: historyLoading ? "#00ff9d" : "#000",
                  fontSize: 12,
                  fontWeight: 700,
                  letterSpacing: "0.1em",
                }}
              >
                {historyLoading ? "Loading…" : "Load history"}
              </button>
            </div>
          </div>
          {historyData?.error && (
            <div
              style={{
                marginBottom: 16,
                padding: "12px 16px",
                background: "rgba(255,51,102,0.1)",
                border: "1px solid rgba(255,51,102,0.3)",
                borderRadius: 6,
                color: "#ff3366",
                fontSize: 12,
              }}
            >
              {historyData.error}
            </div>
          )}
          {historyData && !historyData.error && (
            <div style={{ animation: "fadeUp 0.4s ease" }}>
              {historyData.regression && (
                <div
                  style={{
                    marginBottom: 12,
                    padding: "10px 14px",
                    background: "rgba(239,68,68,0.1)",
                    border: "1px solid rgba(239,68,68,0.3)",
                    borderRadius: 6,
                    fontSize: 12,
                    color: "#ef4444",
                  }}
                >
                  ⚠ TCB regression: previous_svn &gt; current_svn
                </div>
              )}
              {historyData.migration && (
                <div
                  style={{
                    marginBottom: 12,
                    padding: "10px 14px",
                    background: "rgba(245,158,11,0.1)",
                    border: "1px solid rgba(245,158,11,0.3)",
                    borderRadius: 6,
                    fontSize: 12,
                    color: "#f59e0b",
                  }}
                >
                  Migration: {historyData.migration.previous_ppid.slice(0, 12)}… → {historyData.migration.current_ppid.slice(0, 12)}…
                </div>
              )}
              {historyData.records?.length ? (
                <div
                  style={{
                    background: "#0d0d0d",
                    border: "1px solid #141414",
                    borderRadius: 6,
                    overflow: "hidden",
                  }}
                >
                  <table style={{ width: "100%", borderCollapse: "collapse", fontSize: 12 }}>
                    <thead>
                      <tr style={{ borderBottom: "1px solid #1f1f1f" }}>
                        <th style={{ textAlign: "left", padding: "10px 14px", color: "#555", fontWeight: 600 }}>Timestamp</th>
                        <th style={{ textAlign: "left", padding: "10px 14px", color: "#555", fontWeight: 600 }}>TCB SVN</th>
                        <th style={{ textAlign: "left", padding: "10px 14px", color: "#555", fontWeight: 600 }}>Provider</th>
                      </tr>
                    </thead>
                    <tbody>
                      {historyData.records.map((rec, i) => {
                        const isRegressionRow =
                          historyData.regression &&
                          rec.tcb_svn === historyData.regression.previous_svn;
                        return (
                        <tr
                          key={i}
                          style={{
                            borderBottom: "1px solid #141414",
                            background: isRegressionRow ? "rgba(239,68,68,0.12)" : undefined,
                          }}
                        >
                          <td style={{ padding: "10px 14px", color: "#ccc" }}>{rec.timestamp}</td>
                          <td style={{ padding: "10px 14px", color: "#888", fontFamily: "monospace" }}>{rec.tcb_svn}</td>
                          <td style={{ padding: "10px 14px", color: "#00ff9d" }}>{rec.provider ?? "—"}</td>
                        </tr>
                        );
                      })}
                    </tbody>
                  </table>
                </div>
              ) : (
                <p style={{ color: "#555", fontSize: 13 }}>No records for this PPID.</p>
              )}
            </div>
          )}
        </div>
      )}

      {activeTab === "docs" && (
        <div
          style={{ maxWidth: 760, margin: "0 auto", padding: "40px" }}
        >
          <h2
            style={{
              fontFamily: "'Syne', sans-serif",
              color: "#fff",
              marginBottom: 24,
            }}
          >
            Implementation Notes
          </h2>
          {[
            [
              "DCAP Quote Structure",
              "Version 3/4 quotes begin with a 48-byte header. Bytes 0–1 = version (LE), 4–7 = TEE type (0x81 = TDX, 0x00 = SGX), 12–27 = QE Vendor UUID, 28–43 = Platform ID (first 16 bytes of user_data).",
            ],
            [
              "Platform ID / PPID",
              "The PPID uniquely identifies the physical CPU. For DCAP, the Platform ID in user_data binds the quote to hardware. Providers must publish their Platform ID lists for validation.",
            ],
            [
              "Paravisor Verification",
              "In paravirtualized deployments (Azure/OpenHCL, SUSE/COCONUT-SVSM), the paravisor sits between guest and hypervisor. Its build must be reproducible so verifiers can recompute the checksum.",
            ],
            [
              "Threat Model Gap",
              "Current TEEs protect against malicious OS/hypervisor but not physical attacks. Binding the PPID to a known provider registry narrows this gap.",
            ],
          ].map(([title, body]) => (
            <div
              key={String(title)}
              style={{
                marginBottom: 24,
                paddingBottom: 24,
                borderBottom: "1px solid #111",
              }}
            >
              <div
                style={{
                  color: "#00ff9d",
                  fontSize: 12,
                  marginBottom: 8,
                  letterSpacing: "0.08em",
                }}
              >
                {title}
              </div>
              <div
                style={{
                  color: "#666",
                  fontSize: 13,
                  lineHeight: 1.8,
                }}
              >
                {body}
              </div>
            </div>
          ))}
        </div>
      )}

      {activeTab === "api" && (
        <div
          style={{ maxWidth: 760, margin: "0 auto", padding: "40px" }}
        >
          <h2
            style={{
              fontFamily: "'Syne', sans-serif",
              color: "#fff",
              marginBottom: 24,
            }}
          >
            REST API Reference
          </h2>
          <div
            style={{
              marginBottom: 32,
              background: "#0d0d0d",
              border: "1px solid #141414",
              borderRadius: 6,
              overflow: "hidden",
            }}
          >
            <div
              style={{
                padding: "14px 20px",
                borderBottom: "1px solid #141414",
                display: "flex",
                gap: 12,
                alignItems: "center",
              }}
            >
              <span
                style={{
                  fontSize: 11,
                  color: "#000",
                  background: "#00ff9d",
                  padding: "3px 8px",
                  borderRadius: 3,
                  fontWeight: 700,
                }}
              >
                POST
              </span>
              <span
                style={{
                  color: "#fff",
                  fontFamily: "monospace",
                  fontSize: 14,
                }}
              >
                /api/verify
              </span>
            </div>
            <div style={{ padding: "14px 20px" }}>
              <p
                style={{
                  color: "#666",
                  fontSize: 13,
                  margin: "0 0 14px",
                }}
              >
                Verify a DCAP attestation quote.
              </p>
              <div style={{ marginBottom: 12 }}>
                <div
                  style={{
                    fontSize: 10,
                    color: "#444",
                    marginBottom: 6,
                    letterSpacing: "0.1em",
                  }}
                >
                  REQUEST BODY
                </div>
                <pre
                  style={{
                    background: "#080808",
                    border: "1px solid #1a1a1a",
                    borderRadius: 4,
                    padding: "12px",
                    fontSize: 11,
                    color: "#b0ffc8",
                    margin: 0,
                    overflowX: "auto",
                  }}
                >
                  {`{\n  "quote": "<hex string>",\n  "format": "intel_dcap"\n}`}
                </pre>
              </div>
              <div>
                <div
                  style={{
                    fontSize: 10,
                    color: "#444",
                    marginBottom: 6,
                    letterSpacing: "0.1em",
                  }}
                >
                  RESPONSE
                </div>
                <pre
                  style={{
                    background: "#080808",
                    border: "1px solid #1a1a1a",
                    borderRadius: 4,
                    padding: "12px",
                    fontSize: 11,
                    color: "#b0c8ff",
                    margin: 0,
                    overflowX: "auto",
                  }}
                >
                  {`{\n  "valid": true,\n  "tee_type": "Intel DCAP",\n  "platform_id_truncated": "aabbccdd...",\n  "provider_match": { "found": true, "provider": "AWS", "region": "us-east-1" },\n  "status": "Trusted"\n}`}
                </pre>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Footer */}
      <div
        style={{
          borderTop: "1px solid #0f0f0f",
          padding: "24px 40px",
          marginTop: 60,
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <div style={{ fontSize: 11, color: "#2a2a2a" }}>
          TRUSTVERIFY — CONFIDENTIAL COMPUTE ATTESTATION TOOL
        </div>
        <div style={{ fontSize: 11, color: "#2a2a2a" }}>
          Based on Intel DCAP spec •{" "}
          <a
            href="https://arxiv.org/abs/2506.14964"
            style={{ color: "#333" }}
            target="_blank"
            rel="noopener noreferrer"
          >
            TEE Threat Model paper
          </a>
        </div>
      </div>
    </div>
  );
}
