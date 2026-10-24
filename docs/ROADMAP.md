# Telemetry Hub - Engineering Roadmap

This repository hosts the source code and development roadmap for the **Ecological Telemetry Hub**, an edge-to-cloud environmental monitoring platform developed by **Oscar Mora** at **EcoWest Instrument**.

## 🚀 Mission and Architecture (Space / Industrial Grade)

This project is an **Ecological Telemetry Hub** designed from scratch with a *bare-metal* and *zero-cost abstractions* philosophy using **Rust**. The goal is to build an interactive, concurrent, and industrial-grade platform for real-time validation, ingestion, and visualization of environmental sensor data (TinyML, RS485, IoT, Protobuf).

---

## 🗺️ Engineering Roadmap

To ensure relentless quality and comply with industrial standards, the platform's development is divided into the following phases:

### 🟢 Phase 1: Ignition and Rigor (Foundations)
- [x] **Environment Setup:** Titanium foundations with `rust-toolchain.toml`, strict rules in `.rustfmt.toml`, and advanced configurations for `rust-analyzer`.
- [x] **Type Safety and Validation:** Pedantic Clippy rules (`clippy::all`, `clippy::pedantic`, `clippy::nursery`).
- [x] **Router Core:** Base implementation using `axum` on top of `tokio`.
- [x] **Static Server:** Static file serving (CSS/assets) via `tower-http`.
- [x] **Auditing and CI/CD:** Pre-commit hooks, `cargo-audit` (vulnerability detection), `cargo-deny` (licenses), and GitHub Actions pipelines.
- [x] **Basic Testing:** Integration of `cargo test` as a mandatory requirement in the CI/CD pipeline for continuous validation of the core logic.

### 🟡 Phase 2: Static Structure and API Contracts
- [x] **Architectural Modularization:** Clean Architecture pattern, dividing endpoints into logical domains (e.g., `/api/telemetry`, `/portfolio`).
- [x] **SSR Template Engine:** Hierarchy of layouts compiled directly into the binary using `askama` (zero parsing at runtime).
- [x] **Typed Error Handling:** Use of `thiserror` for the domain and `anyhow` for the application, with automatic mapping to HTTP status codes.
- [x] **API Contracts (OpenAPI):** Automatic generation of Swagger/OpenAPI specifications using `utoipa` to document sensor ingestion.
- [x] **Internal Documentation (rustdoc):** Strict use of doc-comments (`///`) on public structs and functions to generate live technical manuals automatically with `cargo doc`.

### 🟠 Phase 3: Telemetry, State, and Resilience
- [x] **Industrial Observability:** Structured and asynchronous logging via `tracing`, aiming to export to **OpenTelemetry (OTel)** (Prometheus/Grafana).
- [x] **Concurrent Shared State:** Safe dependency injection (`State`, `Arc`, `RwLock`) in Axum to handle the global state of the sensor fleet.
- [x] **Graceful Shutdown:** Interception of system signals (SIGINT/SIGTERM) for a safe shutdown without losing telemetry data in flight.
- [x] **Traffic Protection:** `tower` middlewares for *Rate Limiting* and *Timeout*, preventing saturation from burst data sent by faulty hardware.
- [x] **Rigorous Testing (Fuzzing):** Use of `proptest` (Property-Based Testing) against endpoints and logic to test behavior without physical hardware connected.

### 🔵 Phase 4: Persistence and Time-Series Flow
- [x] **Database Layer:** Asynchronous `sqlx` with compile-time query verification.
- [x] **Geospatial/Temporal Storage:** Preparation for PostgreSQL with **PostGIS** and **TimescaleDB** extensions (ideal for environmental time-series modeling and retention).
- [x] **Infrastructure as Code (IaC):** Deterministic spin-up and orchestration of the database using `docker-compose` (local) and Terraform/Nix (for production deployments).
- [x] **Fault Tolerance and Retention:** Implementation of a data retention buffer (Tokio `mpsc` async channels) with exponential backoff (capped at 8s, 5 retries) and critical logging on data loss.
- [x] **Robust IoT Ingestion:** Ultra-fast deserialization (`serde`) and strict payload validation (`validator`) before touching the database.
- [x] **Bidirectional Protocols:** WebSocket streaming for real-time telemetry broadcast to frontend clients. OTA downlink deferred until mTLS auth layer is complete.

### 🟣 Phase 5: Edge & Cloud AI Synergy (Cognitive Digital Twin)
- [ ] **Edge Inference Engine (TinyML):** Integration of Machine Learning models (exported via Edge Impulse) directly into Rust/C++ firmware using `tract` or `ort`.
- [ ] **Real-Time Edge Classification:** Microcontroller-level (ESP32, STM32, Microchip Technology, Texas Instruments) analysis of water/soil quality data for ultra-low latency anomaly detection and alerts.
- [x] **Digital Twin Architecture:** In-memory virtual representation (`LruCache`, 10K cap) of the current state of all deployed hardware nodes.
- [x] **Generative AI Hub Interface:** Integration of the Gemini LLM via Rust Axum backend using `system_instruction` schema for prompt injection protection. Dynamic real-time telemetry reasoning.
- [x] **Agent Terminal UI:** A minimalist terminal web component, acting as the frontend interface to query the Cognitive Digital Twin.

### ⚫ Phase 6: Space Grade (Orbital & Wasm)
- [x] **Reactive Wasm Frontend:** Interactive graphical interface (dashboard) compiled to **WebAssembly** (Leptos), eliminating external JS frameworks.
- [x] **Trunk/Leptos CSR:** Transition of the user interface to Leptos (Wasm) for reactive visualization of high-frequency data without JS overhead.
- [x] **Distroless Containerization:** Creation of ultra-lightweight Docker images (`scratch`, < 15MB) shielded against OS-level vulnerabilities.
- [x] **Cloud-Native Decoupling (Pub/Sub):** Replaced direct Axum mTLS endpoints with a native GCP Pub/Sub Pull Subscriber to ingest Gateway telemetry directly from the cloud infrastructure.
- [x] **Dead-Letter Queue (DLQ):** Implementation of a resilient DLQ (`telemetry_dlq`) in TimescaleDB to catch and persist Protobuf 'Poison Pills' without crashing the worker or entering infinite retry loops.
- [x] **Mega-Schema Convergence (v2.0 - 33 Fields):** Unified 33-field Protobuf schema aligned 1:1 between edge firmware (C/Nanopb) and backend (Rust/prost), covering environmental, particulate counts, optical obstruction diagnostics, virtual sensors (eCO2/bVOC/TVOC), and node operational status.
- [x] **Gateway Health Diagnostics:** Dedicated ingestion pipeline and TimescaleDB hypertable for edge gateway health events (SD card status, heap monitoring, uptime, degradation alerts).
- [x] **Industrial HMI Resilience**: Implemented automatic WebSocket reconnects, "Stale" data attenuation (grayscaling/dimming), and Gruvbox aesthetics.
- [x] **Zero-Trust Authentication (JWT):** Implementation of dual-extraction JWT middleware (Authorization header + Query Parameter) in Axum to isolate HMI telemetry endpoints and ensure strict access control.
- [x] **End-to-End Local Emulator**: Implemented a `dummy_publisher` to inject 21-field Protobuf payloads into the local GCP Pub/Sub emulator to validate full stack connectivity.
- [x] **DLQ Observability (Terminal de Alertas):** Exposed DLQ endpoints via Axum and built a reactive Leptos UI to monitor Poison Pills in real-time.
- [x] **Strict Event ID & Fallback Semantics:** Strict UUID verification on `event_id` in Pub/Sub worker, rejecting invalid IDs as Poison Pills into `telemetry_dlq` instead of generating fallback UUIDs, and explicit `Utc::now()` fallback when `measured_at_ms == 0`.
- [x] **TimescaleDB Mega-Schema Persistence (v2.0 - Fields 22..33):** Added migration `0006_mega_schema_v2_fields.sql` and updated `persist_telemetry` to persist all 33 canonical fields into the `telemetry` hypertable.
- [x] **HMI Diagnostics & Calibration Observability:** Displayed "Calibrando Sensores" badge during BSEC warmup (`is_calibrating`), prominent optical alerts for dirty/blocked laser optics (`is_laser_obstructed`), and network transmission latency tracking (`ingested_at - measured_at`).
- [x] **HMI Global Navigation & SpecSheet Component (Stage 2):** Terminal-style `<SectionNav/>` with live indicator, reusable `<SpecSheet/>` for industrial instrumentation datasheets (`/hardware`, `/hpc`, `/field-research`), and compile-time footer metadata (`env!("CARGO_PKG_VERSION")`, `wasm32-unknown-unknown`).
- [x] **High-Density Metric Readouts & Live Sparklines (Stage 2):** `<MetricReadout/>` with tabular-nums typography, sensor tolerance ranges, 60-sample in-memory historical buffers per device (`VecDeque<f64>`), and reactive SVG sparklines for live Temperature and CO2 trend curves.
- [x] **Executive Engineering Dashboard & SVG Technical Icons (Stage 3):** Replaced all emojis with 1.5px stroke vector icons (microcontroller, optical probe, RS485 bus, mesh grid), structured executive profile with direct GitHub/LinkedIn/EcoWest links and architecture overview.
- [x] **Hardware Systems & Block Diagrams (Stage 3):** Added `/hardware` documenting EcoWest WQM-V100 (optical DO with Sharp Memory LCD), ESP32-P4/C6 dual-chip gateway, and BME688/SCD41/BMV080 nodes with SVG block schematics.
- [x] **AMYNTHAS-HPC Simulation Engine & MathML (Stage 3):** Added `/hpc` with native MathML ecological diffusion-advection-reaction formulation and interactive 2D discrete finite-difference simulation grid in pure Wasm/SVG.
- [x] **Empirical Field Research & Biological Modeling (Stage 3):** Added `/field-research` connecting Colorado semi-arid soil biodiversity and oligochaete taxonomy with empirical sensor pipelines and HPC data assimilation.
---

## 🛠️ Local Development Environment

This repository is configured with the strictest rules. To contribute or spin up the project:

1. **Required tools:** Rust (via `rustup`), Docker (for TimescaleDB), and Trunk (`cargo install trunk`).
2. **Configure environment:**
   ```bash
   cp .env.example .env   # Fill in your GEMINI_API_KEY and DATABASE_URL
   ```
3. **Start the Database and Pub/Sub Emulator:**
   ```bash
   docker-compose up -d
   ```
4. **Build the Wasm Frontend:**
   ```bash
   cd frontend && trunk build
   cd ..
   ```
5. **Run the Axum Server:**
   ```bash
   cargo run -p backend
   ```
6. **Mandatory static analysis (Linting):**
   ```bash
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   ```
7. **Code Formatting:**
   ```bash
   cargo fmt --all -- --check
   ```
8. **Run Tests:**
   ```bash
   cargo test --workspace
   ```
