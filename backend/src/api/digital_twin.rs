// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.
// Proprietary and confidential.

use crate::domain::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

/// Endpoint para leer el estado actual del Gemelo Digital en memoria.
#[utoipa::path(
    get,
    path = "/api/digital-twin",
    responses(
        (status = 200, description = "Estado en tiempo real de todos los biosensores", body = std::collections::HashMap<String, shared::TelemetryPayload>),
    )
)]
#[allow(clippy::unused_async)]
pub async fn get_digital_twin(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let state_map: std::collections::HashMap<String, shared::TelemetryPayload> =
        state.digital_twin.read().await.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

    Ok(Json(state_map))
}
