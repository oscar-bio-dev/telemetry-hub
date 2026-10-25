# Security Policy

## Overview

The **Ecological Telemetry Hub** (`telemetry-hub`) is designed with a *space-grade, industrial zero-trust* architecture. We take the security, integrity, and resilience of our edge-to-cloud environmental telemetry pipeline extremely seriously.

This document outlines our vulnerability disclosure process, supported versions, security threat model, and engineering standards.

---

## Supported Versions

Only the latest active minor release and master/main branch receive security updates and hotfixes:

| Version | Supported          | Status                                |
| ------- | ------------------ | ------------------------------------- |
| 0.8.x   | :white_check_mark: | Current Active Stable (Mega-Schema 2) |
| < 0.8.0 | :x:                | Deprecated / End-of-Life (EOL)        |

---

## Reporting a Vulnerability

**DO NOT file public GitHub issues or discussions for security vulnerabilities.**

If you discover or suspect a security vulnerability, flaw, or data leakage vector within `telemetry-hub`, please report it confidentially using one of the following methods:

### Option A: GitHub Private Vulnerability Reporting (Preferred)
1. Navigate to the [Security Advisories](https://github.com/oscar-bio-dev/telemetry-hub/security/advisories) tab of the repository.
2. Click **"Report a vulnerability"** to submit an encrypted, private report directly to the core maintainer.

### Option B: Direct Security Contact (Email)
Send an email to:
- **Maintainer:** Oscar Mora / EcoWest Instrument
- **Address:** [oscar_bio_dev@outlook.com](mailto:oscar_bio_dev@outlook.com)
- **Subject:** `[SECURITY VULNERABILITY] telemetry-hub - <Brief Summary>`

### What to Include in Your Report
To accelerate triage and remediation, please provide:
1. **Description:** Clear summary of the vulnerability and its potential impact (e.g., authentication bypass, remote panic, privilege escalation, prompt injection).
2. **Affected Component:** Affected crate or file (`backend`, `frontend`, `shared`, Docker configuration, or CI pipeline).
3. **Reproduction Steps:** Step-by-step instructions or automated script to reproduce the issue.
4. **Proof of Concept (PoC):** Minimal reproducible test case, HTTP payload, or Protobuf byte stream.
5. **Mitigation Suggestion:** Any preliminary patches or architecture recommendations (if known).

---

## Response Timeline & SLA

We adhere to the principles of **Coordinated Vulnerability Disclosure (CVD)**:

* **Initial Acknowledgment:** Within **48 hours** of receiving your report.
* **Triage & Severity Assessment:** Within **5 business days**, confirming whether the issue is valid and assigning a severity rating (CVSS v3.1 / v4.0).
* **Patch Development & Testing:** Between **7 to 30 business days**, depending on complexity and scope.
* **Coordinated Public Release:** We coordinate public release of the advisory and patch with the reporter, typically within **30 to 90 days** from initial disclosure.

---

## Scope & Threat Model

### In-Scope Assets & Vectors

We actively encourage research on the following components:

* **Authentication & Authorization:**
  * Zero-Trust mock-login and operator authentication (`POST /api/auth/login`).
  * JWT forgery, signature validation bypasses, or secret leakage (`JWT_SECRET`).
  * Constant-time API key verification mechanisms.
* **HTTP Perimeter & Middleware Pipeline:**
  * CORS bypasses, open-fallback regressions, or origin spoofing.
  * Security response headers (`HSTS`, `CSP`, `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy`).
  * Reverse-proxy rate limiting bypasses (`SmartIpKeyExtractor`, `tower_governor`).
  * Information leakage in internal middleware error handlers (`HandleErrorLayer`).
* **Pub/Sub Worker & Dead-Letter Queue (DLQ):**
  * Malicious Protobuf payloads causing unhandled panics or CPU exhaustion in the background worker.
  * Validation bypasses on `event_id` (UUIDv4) or timestamp fallback injection.
  * Storage exhaustion attacks against `telemetry_dlq` (mitigated by 4KB payload truncation).
* **Cognitive Digital Twin & AI Interface:**
  * Indirect Prompt Injection / Cross-Prompt Injection Attacks (XPIA) via manipulated telemetry metrics in `/api/chat`.
  * Context leakage or model extraction.
* **Memory Safety & Denial of Service:**
  * Memory exhaustion, memory leaks, or unhandled panics (`panic!`) in native Rust crates.
  * Supply-chain vulnerabilities in workspace dependencies (`Cargo.lock`).

### Out-of-Scope

The following items are strictly out-of-scope:

* Attacks requiring physical access to deployed sensor nodes or physical tampering with hardware optics.
* Denial-of-Service attacks targeting network volumetric bandwidth against public cloud infrastructure (Google Cloud Run / TimescaleDB Cloud).
* Social engineering, phishing, or physical attacks against maintainers or contributors.
* Issues regarding third-party development emulators (e.g., Google Cloud SDK Pub/Sub emulator) when running purely in isolated local developer environments without external network exposure.
* Publicly known dependencies with documented exceptions in `audit.toml` that have no actionable exploit path in this architecture.

---

## Security Architecture & Best Practices

The platform maintains defense-in-depth through strict space-grade principles:

1. **Zero-Panic Mandate:** Production code strictly avoids `.unwrap()` and `.expect()` on dynamic or runtime configurations, propagating errors cleanly via `Result<T, E>`.
2. **Fail-Closed Secrets:** If `JWT_SECRET` is missing or insufficient (< 32 bytes), the application refuses to start instead of falling back to default keys.
3. **CORS Fail-Closed:** If `CORS_ORIGIN` is unset, cross-origin requests are blocked by default.
4. **Minimal Attack Surface:**
   - Scratch Distroless Docker image (< 15MB) running as non-root user (`nobody` `65534:65534`).
   - Swagger UI disabled in production unless explicitly enabled via `ENABLE_SWAGGER=true`.
   - CI workflow actions pinned to immutable commit SHA hashes.
5. **Continuous Automated Auditing:**
   - Every commit and pull request runs `cargo audit`, `cargo clippy` (`-D warnings`), and `cargo test`.
   - Pre-commit hooks enforce IP header integrity and static analysis before staging.
