// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.
use crate::domain::state::AppState;
use axum::{
    body::Bytes,
    extract::{Request, State},
    http::{header, HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use jsonwebtoken::{decode, encode, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Estructura de los Claims del JWT
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (quien es el usuario, ej. "operator-admin")
    pub sub: String,
    /// Expiration time (UNIX timestamp)
    pub exp: usize,
    /// Role for authorization ("admin")
    pub role: String,
}

/// Request DTO para el login de operador
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    /// Clave secreta de operador (si no se envía por la cabecera x-api-key)
    pub api_key: String,
}

/// Response DTO con el token JWT de operador
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LoginResponse {
    /// JWT Bearer Token
    pub token: String,
    /// Duración de validez en segundos (900s = 15m)
    pub expires_in: usize,
}

/// Compara dos secuencias de bytes en tiempo constante para evitar ataques de temporización.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let max_len = a.len().max(b.len());
    let mut diff = 0u8;
    for i in 0..max_len {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        diff |= x ^ y;
    }
    diff == 0 && a.len() == b.len()
}

/// Endpoint para autenticar operadores autorizados y generar un JWT con permisos de admin.
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body(content = Option<LoginRequest>, description = "JSON opcional con api_key si no se envía la cabecera x-api-key"),
    responses(
        (status = 200, description = "JWT Token de operador generado exitosamente", body = LoginResponse),
        (status = 401, description = "Credenciales de operador inválidas"),
        (status = 503, description = "Autenticación de operador no configurada en el servidor")
    )
)]
#[allow(clippy::unused_async, clippy::cast_possible_truncation)]
pub async fn operator_login(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    let admin_key = match std::env::var("ADMIN_API_KEY") {
        Ok(k) if !k.trim().is_empty() => k,
        _ => {
            tracing::error!("ADMIN_API_KEY no está configurada en las variables de entorno");
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                "Operator authentication not configured on server".to_string(),
            ));
        }
    };

    // 1. Extraer credencial: primero desde header `x-api-key`, luego desde body JSON
    let key_from_header =
        headers.get("x-api-key").and_then(|v| v.to_str().ok()).map(ToString::to_string);

    let key_from_body = serde_json::from_slice::<LoginRequest>(&body).ok().map(|r| r.api_key);

    let Some(provided_key) = key_from_header.or(key_from_body) else {
        tracing::warn!("Intento de login sin clave de operador (x-api-key ausente)");
        return Err((StatusCode::UNAUTHORIZED, "Missing API key".to_string()));
    };

    // 2. Comparación en tiempo constante contra ADMIN_API_KEY
    if !constant_time_eq(provided_key.as_bytes(), admin_key.as_bytes()) {
        tracing::warn!("Intento fallido de login con clave de operador inválida");
        return Err((StatusCode::UNAUTHORIZED, "Invalid API key".to_string()));
    }

    // 3. Emisión de JWT con TTL de 15 minutos (900 segundos)
    let expiration =
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as usize + 900;

    let claims =
        Claims { sub: "operator-admin".to_owned(), exp: expiration, role: "admin".to_owned() };

    let token = encode(&Header::default(), &claims, &state.jwt_encoding_key).map_err(|e| {
        tracing::error!("Error al firmar JWT de operador: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Error interno".to_string())
    })?;

    tracing::info!("Login de operador exitoso. JWT emitido con TTL de 15m.");
    Ok(Json(LoginResponse { token, expires_in: 900 }))
}

/// Middleware que verifica el JWT estricto en el header Authorization Bearer
pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let mut token = None;

    // Check Authorization Bearer Header ONLY (No query parameter leak)
    if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(stripped) = auth_str.strip_prefix("Bearer ") {
                token = Some(stripped.to_string());
            }
        }
    }

    let Some(token) = token else {
        tracing::warn!("Acceso denegado: Token de autorización no proporcionado");
        return Err(StatusCode::UNAUTHORIZED);
    };

    // Validar el Token y verificar permisos RBAC
    match decode::<Claims>(&token, &state.jwt_decoding_key, &Validation::default()) {
        Ok(token_data) => {
            if token_data.claims.role != "admin" {
                tracing::warn!(
                    "Acceso prohibido: Rol '{}' no autorizado para recursos administrativos",
                    token_data.claims.role
                );
                return Err(StatusCode::FORBIDDEN);
            }
            tracing::debug!("Token de operador válido para subject: {}", token_data.claims.sub);
            Ok(next.run(req).await)
        }
        Err(e) => {
            tracing::warn!("Acceso denegado: Token inválido - {}", e);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_eq_matches() {
        assert!(constant_time_eq(b"secret_key_12345", b"secret_key_12345"));
    }

    #[test]
    fn test_constant_time_eq_mismatch_same_len() {
        assert!(!constant_time_eq(b"secret_key_12345", b"secret_key_1234X"));
    }

    #[test]
    fn test_constant_time_eq_different_len() {
        assert!(!constant_time_eq(b"secret_key_12345", b"secret_key_12345_extra"));
        assert!(!constant_time_eq(b"short", b"longer_secret"));
    }
}
