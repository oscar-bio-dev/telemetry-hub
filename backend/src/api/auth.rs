// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.
use crate::domain::state::AppState;
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use jsonwebtoken::{decode, encode, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Estructura de los Claims del JWT
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (quien es el usuario, ej. "hmi-dashboard")
    pub sub: String,
    /// Expiration time
    pub exp: usize,
    /// Role for authorization
    pub role: String,
}

/// Endpoint para generar un token temporal para el HMI (Mock Login)
#[utoipa::path(
    post,
    path = "/api/auth/mock-login",
    responses(
        (status = 200, description = "JWT Token generado exitosamente")
    )
)]
#[allow(clippy::unused_async, clippy::cast_possible_truncation)]
pub async fn mock_login(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let expiration =
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize + 3600; // 1 hour

    let claims =
        Claims { sub: "hmi-dashboard".to_owned(), exp: expiration, role: "admin".to_owned() };

    let token = encode(&Header::default(), &claims, &state.jwt_encoding_key).map_err(|e| {
        tracing::error!("Error al firmar JWT: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Error interno".to_string())
    })?;

    tracing::info!("Mock Login exitoso. JWT generado para hmi-dashboard.");
    Ok(Json(serde_json::json!({
        "token": token,
        "expires_in": 3600
    })))
}

/// Middleware que verifica el JWT
pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let mut token = None;

    // 1. Check Authorization Bearer Header
    if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(stripped) = auth_str.strip_prefix("Bearer ") {
                token = Some(stripped.to_string());
            }
        }
    }

    // 2. Check Query Parameters for `token=` (for WebSockets)
    if token.is_none() {
        if let Some(query) = req.uri().query() {
            let parsed_query: HashMap<String, String> =
                serde_urlencoded::from_str(query).unwrap_or_default();
            if let Some(t) = parsed_query.get("token") {
                token = Some(t.clone());
            }
        }
    }

    let Some(token) = token else {
        tracing::warn!("Acceso denegado: Token no proporcionado");
        return Err(StatusCode::UNAUTHORIZED);
    };

    // Validar el Token
    match decode::<Claims>(&token, &state.jwt_decoding_key, &Validation::default()) {
        Ok(token_data) => {
            tracing::debug!("Token válido para subject: {}", token_data.claims.sub);
            Ok(next.run(req).await)
        }
        Err(e) => {
            tracing::warn!("Acceso denegado: Token inválido - {}", e);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}
