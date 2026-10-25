// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.

/// Determina de forma segura la URL base para peticiones HTTP a la API.
/// Nunca genera panic; si `web_sys::window()` no está disponible (ej. SSR o entornos sin DOM),
/// recurre a un valor por defecto seguro.
#[must_use]
pub fn api_base_url() -> String {
    if let Some(window) = web_sys::window() {
        let location = window.location();
        if let (Ok(host), Ok(protocol)) = (location.host(), location.protocol()) {
            if host.contains("localhost") || host.contains("127.0.0.1") {
                return "http://127.0.0.1:3000".to_string();
            }
            return format!("{protocol}//{host}");
        }
    }
    "http://127.0.0.1:3000".to_string()
}

/// Determina de forma segura la URL base para conexiones WebSocket.
#[must_use]
pub fn ws_base_url() -> String {
    if let Some(window) = web_sys::window() {
        let location = window.location();
        if let (Ok(host), Ok(protocol)) = (location.host(), location.protocol()) {
            let ws_protocol = if protocol == "https:" { "wss:" } else { "ws:" };
            if host.contains("localhost") || host.contains("127.0.0.1") {
                return "ws://127.0.0.1:3000".to_string();
            }
            return format!("{ws_protocol}//{host}");
        }
    }
    "ws://127.0.0.1:3000".to_string()
}

/// Construye una URL completa para un endpoint HTTP de la API sin riesgo de panic.
#[must_use]
pub fn api_url(endpoint: &str) -> String {
    let base = api_base_url();
    let clean_endpoint = endpoint.trim_start_matches('/');
    format!("{base}/{clean_endpoint}")
}

/// Construye una URL completa para un endpoint WebSocket sin riesgo de panic.
#[must_use]
pub fn ws_url(endpoint: &str) -> String {
    let base = ws_base_url();
    let clean_endpoint = endpoint.trim_start_matches('/');
    format!("{base}/{clean_endpoint}")
}
