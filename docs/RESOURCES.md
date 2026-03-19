# Resources for PPID Verification Dashboard

Curated references for TEE attestation, PPID verification, and related work. Last updated: March 2025.

---

## Core Papers & Theory

| Resource | URL | Description |
|----------|-----|-------------|
| **Narrowing the Gap between TEEs Threat Model and Deployment Strategies** (Rezabek et al.) | [arXiv:2506.14964](https://arxiv.org/abs/2506.14964) | Your primary reference. Identifies gap between attestation and threat model; proposes PPID to strengthen provider binding. |
| **Proof of Cloud: Data Center Execution Assurance for Confidential VMs** | [arXiv:2510.12469](https://arxiv.org/abs/2510.12469) | DCEA framework; binds CVM attestation to platform TPM; relates to Intel POE. |
| **Platform Ownership Endorsements (POE)** | [Intel Developer](https://www.intel.com/content/www/us/en/developer/articles/technical/software-security-guidance/technical-documentation/platform-ownership-endorsements.html) | Verifies who physically possesses hardware; adds protection against physical attacks. |

---

## Related Projects

| Project | URL | What It Offers |
|---------|-----|-----------------|
| **Proof of Cloud** | [proofofcloud.org](https://www.proofofcloud.org/) | Public registry of hardware IDs (PPID, Chip ID) to verified locations; Verify feature; alliance-based. |
| **Automata DCAP Dashboard** | [docs.ata.network/dcap-dashboard](https://docs.ata.network/dcap-dashboard) | Collateral lifecycle management for Intel SGX/TDX; quote verification; no PPID lookup. |
| **Confidential Computing Consortium** | [confidentialcomputing.io](https://confidentialcomputing.io/) | Industry standards; Attestation SIG; RA-TLS harmonisation. |
| **CCC Attestation** | [github.com/CCC-Attestation](https://github.com/CCC-Attestation) | Open-source attestation implementations. |
| **Confidential Containers / Trustee** | [confidentialcontainers.org/docs/attestation](https://confidentialcontainers.org/docs/attestation/) | KBS, Attestation Service, RVPS; verifies attestations before releasing secrets. |

---

## Intel DCAP & Quote Verification

| Resource | URL | Description |
|----------|-----|-------------|
| **SGX-TDX-DCAP-QuoteVerificationLibrary** | [GitHub](https://github.com/intel/SGX-TDX-DCAP-QuoteVerificationLibrary) | Quote verification for SGX and TDX; reference implementation. |
| **Intel SGX DCAP (Data Center Attestation Primitives)** | [GitHub](https://github.com/intel/SGXDataCenterAttestationPrimitives) | Core DCAP libraries; quote generation and verification. |
| **ECDSA Quote Library API Reference** | [Intel docs](https://download.01.org/intel-sgx/latest/dcap-latest/linux/docs/Intel_SGX_ECDSA_QuoteLibReference_DCAP_API.pdf) | Quote structure, report_data, Platform ID. |
| **Quote Verification with Intel SGX DCAP** | [Intel Article](https://www.intel.com/content/www/us/en/developer/articles/technical/quote-verification-attestation-with-intel-sgx-dcap.html) | End-to-end verification flow. |
| **Intel Trust Authority (PLI, TEE)** | [docs.trustauthority.intel.com](https://docs.trustauthority.intel.com/main/articles/articles/tsc/pli-intro.html) | Platform Lifecycle Assurance; TDX attestation services. |
| **Intel DCAP Multi-Package RA** | [cc-enabling.trustedservices.intel.com](https://cc-enabling.trustedservices.intel.com/intel-dcap-mp-ra/01/introduction/) | Remote attestation for multi-package platforms. |

---

## AMD SEV-SNP

| Resource | URL | Description |
|----------|-----|-------------|
| **AWS EC2 SEV-SNP Attestation** | [AWS Docs](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/snp-attestation.html) | Attest EC2 instances with AMD SEV-SNP. |
| **AMD SEV-SNP (Edgeless Contrast)** | [docs.edgeless.systems/contrast](https://docs.edgeless.systems/contrast/1.7/architecture/snp) | Architecture overview; attestation report structure. |
| **Microsoft CCF - AMD SEV-SNP** | [microsoft.github.io/CCF](https://microsoft.github.io/CCF/main/operations/platforms/snp.html) | SNP integration in CCF. |
| **IBM: Remote attestation of CVMs with ephemeral vTPMs** | [IBM Research](https://research.ibm.com/publications/remote-attestation-of-confidential-vms-using-ephemeral-vtpms) | vTPM-based attestation. |

---

## Rust Libraries & Crates

| Crate/Project | URL | Use Case |
|---------------|-----|----------|
| **tdx-quote** | [crates.io](https://crates.io/crates/tdx-quote) / [docs.rs](https://docs.rs/tdx-quote/latest/tdx_quote/) | Parse and verify Intel TDX quotes (v4/v5); PCK chain; mock feature. |
| **sev-snp-utilities** | [crates.io](https://crates.io/crates/sev-snp-utilities) | AMD SEV-SNP attestation reports; cert chain verification. |
| **sev-snp-utils** | [docs.rs](https://docs.rs/crate/sev-snp-utils/latest) | Earlier AMD SEV-SNP utilities. |
| **Cosmian tee-tools** | [GitHub](https://github.com/Cosmian/tee-tools) | TEE tooling and attestation utilities. |
| **virtee snphost** | [GitHub](https://github.com/virtee/snphost) | Administrative utility for SEV-SNP. |

---

## Python / Other Languages

| Library | URL | Description |
|---------|-----|-------------|
| **intel-sgx-ra** (Cosmian) | [PyPI](https://pypi.org/project/intel-sgx-ra/) | Python remote attestation; `sgx-ra-verify`; PCCS and Azure Attestation. |

---

## Cloud Provider Attestation Docs

| Provider | URL |
|----------|-----|
| **Google Confidential Computing** | [cloud.google.com/confidential-computing](https://docs.cloud.google.com/confidential-computing/confidential-vm/docs/attestation) |
| **AWS EC2 SNP** | [docs.aws.amazon.com](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/snp-attestation.html) |
| **Azure Attestation** | [learn.microsoft.com/azure/attestation](https://learn.microsoft.com/en-us/azure/attestation/) |

---

## Community & Events

| Resource | URL |
|----------|-----|
| **Encode Club - Shape Rotator Hackathon** | [encodeclub.com](https://www.encodeclub.com/programmes/shape-rotator-virtual-hackathon/) |
| **Flashbots LooseSEAL (TDX)** | [collective.flashbots.net](https://collective.flashbots.net/t/loose-seal-enabling-crash-tolerant-tdx-applications/4243) |
| **SPIFFE / SPIRE SEV-SNP Plugin** | [GitHub Issue #4469](https://github.com/spiffe/spire/issues/4469) |

---

## Quick Reference: Where to Find What

| Need | Go To |
|------|-------|
| PPID in Intel quotes | ECDSA Quote Library API; first 16B of user_data / report_data |
| Chip ID in AMD reports | sev-snp-utilities; attestation report structure |
| Full quote verification | Intel QuoteVerificationLibrary; Cosmian tdx-quote |
| Provider registry design | Proof of Cloud; DCEA paper |
| Threat model context | Rezabek arXiv:2506.14964 |
| Physical security / POE | Intel Platform Ownership Endorsements |

---

*Add more resources as you discover them. Pull requests welcome.*
