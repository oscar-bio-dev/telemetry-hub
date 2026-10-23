# Workspace Agent Rules & Constraints (telemetry-hub)

This document defines the strict, industrial-grade behavioral rules, architectural guidelines, and repository governance standards that any AI agent must follow when interacting with this workspace. This repository (`telemetry-hub`) is exclusively dedicated to **Rust, Frontend (Wasm), Backend, Telemetry, Databases, and Web HMI**.

> **Enforcement Levels:** Rules use the following classifications:
> - 🔴 **MANDATORY** — Hard-blocking. Violation prevents merge or deployment.
> - 🟡 **RECOMMENDED** — Strongly encouraged. Deviation requires written justification in the PR.
> - ⚪ **GUIDELINE** — Best practice. Follow unless context demands otherwise.

## 1. Industrial Space-Grade Systems Engineering

- 🔴 **Rust Exclusivity**: All core logic, backend services, and web frontends must be written in Rust. Adhere to a *bare-metal* and *zero-cost abstractions* mindset.
- 🔴 **Type Safety & Data Integrity**: 
  - Follow the "Parse, don't validate" paradigm using the `validator` crate.
  - Ensure lossless error propagation using `thiserror`.
  - Maintain the Hybrid Type Strategy: `f32` for Protobuf wire payloads (DTOs), and explicit casting to `f64` for Domain models and Database analytics.
- 🔴 **Security & Stability**: 
  - 0 `unsafe` blocks unless explicitly authorized by a human principal engineer. 
  - No `panic!` or `unwrap()` in production code; handle all errors gracefully via `Result`.
  - Enforce poison-pill isolation: Any malformed telemetry must be immediately routed to a Dead-Letter Queue (DLQ) in PostgreSQL without crashing the async workers.

## 2. Cloud-Native & Decoupled Architecture

- 🔴 **Ingestion Pipeline**: The backend does NOT expose direct TCP/mTLS ingestion ports to edge devices. It acts purely as a Pull Subscriber to Google Cloud Pub/Sub.
- 🔴 **Idempotency**: All database insertions (TimescaleDB) must handle conflicts gracefully (e.g., `ON CONFLICT DO NOTHING`) using a composite unique key (`event_id`, `measured_at`).
- 🔴 **Stateless Async Workers**: Use `tokio` for all asynchronous workloads. Do not block the executor thread with heavy computations or synchronous I/O.

## 3. Industrial HMI & Web Standards (Leptos/Wasm)

- 🔴 **Extreme Fault Tolerance**: The UI must never freeze. Use WebSockets with automatic backoff reconnection logic. Handle network loss and component unmounting gracefully.
- 🟡 **Terminal/Industrial Aesthetics**: Web applications must lean towards a sleek, functional industrial terminal aesthetic using the **Gruvbox Hard Dark** color palette. Avoid bloated whitespace.
- 🔴 **Telemetry Staleness**: The frontend MUST visually represent the "age" of data. Telemetry older than 5 seconds must be visually attenuated (e.g., dimmed, grayscale) to indicate a stale state to the operator.
- 🔴 **Modern & Lightweight**: Use **Leptos (WASM/CSR)** for the frontend. Rely on native CSS variables and modern HTML5 APIs. Avoid bloated JavaScript frameworks.

## 4. GitHub Workflow & Pull Request Policies

### 4.1 Branch Protection (Enforced via GitHub Settings)

- 🔴 `main` is strictly protected. **No direct pushes to main**.
- 🔴 **Required status checks must pass** before merge (see §5 for the exact list).
- 🔴 **Require branch to be up-to-date** before merging.
- 🔴 **Minimum 1 approval** required. Changes to critical paths (§4.5) require 2 approvals.
- 🔴 **Dismiss stale approvals** on new commits.
- 🔴 **Require conversation resolution** before merge.
- 🔴 **Disable force-push** and **branch deletion** on `main`.
- 🔴 **Squash merge only**. No merge commits or rebase merges.

### 4.2 Branching Strategy

- 🔴 Use semantic branch names: `feat/...`, `fix/...`, `docs/...`, `refactor/...`, `ci/...`.
- 🟡 Delete merged branches within 7 days.

### 4.3 Pull Request Rules

- 🔴 **1 PR = 1 Propósito**: Each PR must address exactly ONE of: a feature, a bugfix, a bounded refactor, a documentation change, or a CI/infra change. The following combinations are **explicitly prohibited**:
  - Feature + unrelated refactor
  - Bugfix + cosmetic renames
  - Functional change + bulk reformatting
- 🔴 **PR Titles**: Must strictly follow Conventional Commits (e.g., `feat(ui): add telemetry staleness indicator`).
- 🔴 **Linked Issue**: Every PR must reference an Issue using `Closes #...` or `Refs #...`. PRs without a linked issue must justify the omission in the description.
- 🔴 **PR Description Template** (all fields mandatory):
  ```
  ## Context / Problem
  ## Solution
  ## Risks & Breaking Changes
  ## Validation Evidence
  - Commands executed:
  - Test results:
  - Screenshots (if UI change):
  - Migration notes (if schema/API change):
  ## Impact on Docs / Changelog
  ```
- 🔴 **Merge Blocks**: Do not merge if CI fails, there are compiler warnings, documentation is missing, conversations are unresolved, or reviews are pending.

### 4.4 Draft PR Policy

- 🟡 **Draft PRs** should be used for work-in-progress that needs early CI feedback.
- 🔴 **Draft PRs must NOT be merged**. Convert to "Ready for Review" first.
- ⚪ Use `[WIP]` prefix in draft titles for visibility.

### 4.5 CODEOWNERS & Critical Path Reviews

Changes to the following paths require **2 approvals**, including at least one from a domain owner:

| Critical Path | Required Reviewer Domain |
|---|---|
| `backend/migrations/` | Database / Schema Owner |
| `backend/src/infrastructure/pubsub.rs` | Ingestion Pipeline Owner |
| `shared/src/lib.rs` (Protobuf DTOs) | Data Contract Owner |
| `proto/*.proto` | Data Contract Owner |
| `.github/workflows/` | CI/CD / DevSecOps Owner |
| `docker-compose.yml`, `Dockerfile` | Infrastructure Owner |
| `.agents/AGENTS.md` | Principal Engineer |

### 4.6 Exception & Hotfix Process

- 🟡 **Hotfix branches** (`hotfix/...`) may bypass the standard PR flow **only** under the following conditions:
  1. A production incident is actively impacting users or data integrity.
  2. The Principal Engineer (or designated on-call lead) explicitly authorizes the bypass.
  3. The hotfix targets a single, well-scoped fix.
- 🔴 **Post-Hotfix Requirements**: Within 48 hours of a hotfix merge, a follow-up issue must be created documenting:
  - Root cause analysis
  - What was bypassed (reviews, tests, etc.)
  - Remediation plan to prevent recurrence

## 5. CI/CD & Security Compliance

### 5.1 Hard-Blocking CI Checks (per PR)

These must ALL pass before merge is allowed:

| Check | Command | Blocking? |
|---|---|---|
| Formatting | `cargo fmt --check` | 🔴 Hard-block |
| Linting | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | 🔴 Hard-block |
| Unit Tests | `cargo test --workspace` | 🔴 Hard-block |
| Vulnerability Audit | `cargo audit` | 🔴 Hard-block |
| PR Title Format | Conventional Commits validation | 🔴 Hard-block |

### 5.2 Advisory Checks

| Check | Command | Blocking? |
|---|---|---|
| License Compliance | `cargo deny check` | 🟡 Advisory |
| Doc Coverage | `cargo doc --workspace --no-deps` | 🟡 Advisory |

### 5.3 Security Policies

- 🔴 **No `.env` commits**. Use GitHub Secrets for CI environments and Secret Manager for production.
- 🔴 Secret scanning and Dependabot must be enabled and alerts resolved within 72 hours.
- 🔴 GitHub Actions must be pinned to audited SHA commits (no mutable tags like `@v4`).
- 🔴 Release workflows must declare explicit `permissions` blocks with minimum required scopes.

## 6. Repository Hygiene & Planning

- 🔴 **Issue Tracking**: All work must be tracked through an Issue or documented task. Use consistent labels (`bug`, `feature`, `documentation`, `security`, `priority:high`).
- 🔴 **Documentation Sync**: `README.md`, `docs/ROADMAP.md`, `CHANGELOG.md`, and architecture docs must be kept strictly synchronized with the shipped code state.
- 🔴 **Releases**: Follow Semantic Versioning (SemVer), tag releases explicitly, and document breaking changes, fixes, and features concisely in `CHANGELOG.md`.
- 🔴 **Proprietary IP**: Ensure every newly created source code file (`.rs`) contains the official EcoWest Instrument copyright and confidentiality header.
- 🟡 **Stale PR Hygiene**: PRs with no activity for 14 days should be pinged. PRs with no activity for 30 days should be closed with a summary comment.