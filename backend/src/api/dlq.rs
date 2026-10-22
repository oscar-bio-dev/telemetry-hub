// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.
// Proprietary and confidential.

use crate::domain::state::AppState;
use axum::{extract::State, routing::get, Json, Router};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::Row;

/// DTO for a DLQ record
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DlqRecordDto {
    /// ID of the DLQ record
    pub id: String,
    /// Timestamp of ingestion
    pub ingested_at: DateTime<Utc>,
    /// Base64 encoded payload
    pub raw_payload_base64: String,
    /// Reason for failure
    pub error_reason: String,
    /// Gateway ID
    pub gateway_id: Option<String>,
    /// Event ID
    pub event_id: Option<String>,
}

/// Returns the router for DLQ endpoints.
pub fn router() -> Router<AppState> {
    Router::new().route("/recent", get(get_recent_dlq))
}

/// Retrieves the most recent 50 Poison Pills from the DLQ.
#[utoipa::path(
    get,
    path = "/api/dlq/recent",
    responses(
        (status = 200, description = "Recent DLQ items", body = Vec<DlqRecordDto>)
    ),
    security(
        ("jwt_auth" = [])
    )
)]
async fn get_recent_dlq(State(state): State<AppState>) -> Json<Vec<DlqRecordDto>> {
    let records = sqlx::query(
        r"
        SELECT id, ingested_at, raw_payload, error_reason, gateway_id, event_id
        FROM telemetry_dlq
        ORDER BY ingested_at DESC
        LIMIT 50
        ",
    )
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    let dtos = records
        .into_iter()
        .map(|r| DlqRecordDto {
            id: r.get::<uuid::Uuid, _>("id").to_string(),
            ingested_at: r.get("ingested_at"),
            raw_payload_base64: STANDARD.encode(r.get::<&[u8], _>("raw_payload")),
            error_reason: r.get("error_reason"),
            gateway_id: r.get("gateway_id"),
            event_id: r.get::<Option<uuid::Uuid>, _>("event_id").map(|u| u.to_string()),
        })
        .collect();

    Json(dtos)
}
