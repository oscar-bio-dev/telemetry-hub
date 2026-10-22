// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.
// Proprietary and confidential.

//! Endpoints para Kubernetes Probes (Liveness & Readiness)

use crate::domain::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "El servidor está vivo")
    )
)]
/// Endpoint de liveness probe para Kubernetes.
/// Devuelve 200 OK si el servidor HTTP está respondiendo.
#[allow(clippy::unused_async)]
pub async fn liveness_probe() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

#[utoipa::path(
    get,
    path = "/ready",
    responses(
        (status = 200, description = "El servidor y la base de datos están listos"),
        (status = 503, description = "Base de datos desconectada")
    )
)]
/// Endpoint de readiness probe para Kubernetes.
/// Devuelve 200 OK si el servidor, la base de datos, y el Pub/Sub worker están listos.
pub async fn readiness_probe(State(state): State<AppState>) -> impl IntoResponse {
    use std::sync::atomic::Ordering;

    let db_ok = sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(&state.db_pool).await.is_ok();

    let pubsub_ok = state.pubsub_ready.load(Ordering::SeqCst);

    if db_ok && pubsub_ok {
        (StatusCode::OK, Json(serde_json::json!({ "status": "up" })))
    } else {
        tracing::error!("Readiness probe falló: db_ok={}, pubsub_ok={}", db_ok, pubsub_ok);
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "down",
                "db_ready": db_ok,
                "pubsub_ready": pubsub_ok
            })),
        )
    }
}
