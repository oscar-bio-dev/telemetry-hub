# ── Stage 1: Builder (Frontend Wasm + Backend Musl) ──────────────────────────
FROM rust:1.85-alpine AS builder

# Install build dependencies for musl static linking and OpenSSL
RUN apk add --no-cache musl-dev pkgconfig openssl-dev perl make

# Add Wasm target for Leptos frontend
RUN rustup target add wasm32-unknown-unknown
# Install Trunk securely via cargo (locked dependencies)
RUN cargo install trunk --version 0.20.1 --locked

WORKDIR /app

# Copy the entire workspace
COPY . .

# Build the Frontend (Wasm)
WORKDIR /app/frontend
RUN trunk build --release

# Build the Backend (Musl static binary)
WORKDIR /app
RUN cargo build --bin backend --release --target x86_64-unknown-linux-musl

# ── Stage 2: Scratch (Zero-OS footprint, < 15MB) ────────────────────────────
FROM scratch

WORKDIR /app

# Copy the statically-linked backend binary
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/backend /app/backend/server

# Copy the frontend dist (served by the backend via ServeDir)
COPY --from=builder /app/frontend/dist /app/frontend/dist

# Copy SQL migrations (auto-applied on startup via sqlx::migrate!)
COPY --from=builder /app/backend/migrations /app/backend/migrations

# Expose Public Port (:3000)
EXPOSE 3000

# Environment defaults (override via docker run -e or Cloud Run)
ENV PORT=3000
ENV APP_HOST=0.0.0.0

WORKDIR /app/backend
USER 65534:65534
ENTRYPOINT ["/app/backend/server"]
