# Contributing to telemetry-hub

First off, thank you for considering contributing to the `telemetry-hub` platform! We aim for space-grade, industrial-level code quality.

## Development Process

1. **Fork the Repository:** Create your own branch from `main`.
2. **Environment Setup:**
    - Install the Rust toolchain (we use stable, pinned via `rust-toolchain.toml`).
    - Install Trunk for Wasm builds: `cargo install trunk`.
    - Run `docker-compose up -d` to start the TimescaleDB/PostGIS database and GCP Pub/Sub emulator.
    - Copy `.env.example` to `.env` and fill in your `DATABASE_URL` and `GEMINI_API_KEY`.
    - Migrations run automatically on backend startup via `sqlx::migrate!()`.
3. **Build the Frontend:** `cd frontend && trunk build && cd ..`
4. **Commit Standards:** We strictly follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).
    - Example: `feat(telemetry): add support for BMV080 particulate matter sensor`
    - Example: `fix(api): resolve memory leak in digital twin`
    - Scopes: `telemetry`, `api`, `security`, `frontend`, `docs`, `ci`, `core`

## Quality Gates (Pre-requisites for PR)

Before opening a Pull Request, you MUST ensure the following checks pass locally:

- **Formatting:** Code must be formatted using `cargo fmt`.
  ```bash
  cargo fmt --all
  ```
- **Linting:** We enforce pedantic clippy rules. There must be zero warnings.
  ```bash
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  ```
- **Tests:** All unit and property-based tests must pass.
  ```bash
  cargo test --workspace
  ```
- **Security:** Run `cargo audit` to ensure no known vulnerabilities are introduced.

> **Note:** Pre-commit hooks are configured to automatically run `fmt`, `clippy`, `test`, and IP header verification. A commit will be rejected if any of these checks fail.

## Architectural Guidelines
- **Type Safety over Validation:** Parse, don't validate. Use Rust's type system to represent state (newtypes with validated constructors like `Temperature`, `Iaq`, `Pm2_5`).
- **Zero-Cost Abstractions:** Keep hot-paths (telemetry ingestion) allocation-free where possible.
- **Hybrid f32/f64:** Protobuf DTOs use `f32` (hardware wire type). Domain models and DB use `f64` for analytical precision.
- **No Unsafe:** Unless absolutely necessary and heavily documented.
- **No Panics:** All `.unwrap()` and `.expect()` calls are prohibited in production code. Use `Result<>` propagation.
- **Copyright Header:** Every `.rs` file must contain the EcoWest Instrument copyright and confidentiality header.
