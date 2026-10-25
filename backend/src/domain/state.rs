// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.
//
// This file is part of the telemetry-hub platform.
// Unauthorized copying of this file, via any medium, is strictly prohibited.
// Proprietary and confidential.

//! Memoria compartida del servidor (Gemelo Digital).

use lru::LruCache;
use shared::TelemetryPayload;
use sqlx::PgPool;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use std::sync::atomic::AtomicBool;

const TWIN_CAPACITY: NonZeroUsize = match NonZeroUsize::new(10_000) {
    Some(v) => v,
    None => panic!("Invalid capacity"),
};

/// Estado global de la aplicación inyectado en las rutas de Axum.
#[derive(Clone)]
pub struct AppState {
    /// Buffer en RAM de las últimas lecturas de telemetría de cada sensor.
    /// Funciona como el gemelo digital en tiempo real de la flota física.
    /// Limitado a 10,000 entradas para evitar `DoS` por agotamiento de RAM.
    pub digital_twin: Arc<RwLock<LruCache<String, TelemetryPayload>>>,
    /// Pool de conexiones a la base de datos (`TimescaleDB`).
    pub db_pool: PgPool,
    /// Canal broadcast para notificar telemetría en tiempo real a `WebSockets`.
    pub tx_ws: broadcast::Sender<TelemetryPayload>,
    /// Indicador de salud (Readiness) del cliente de Pub/Sub.
    pub pubsub_ready: Arc<AtomicBool>,
    /// Llave criptográfica para firmar JWTs
    pub jwt_encoding_key: Arc<jsonwebtoken::EncodingKey>,
    /// Llave criptográfica para decodificar y validar JWTs
    pub jwt_decoding_key: Arc<jsonwebtoken::DecodingKey>,
    /// Cliente HTTP reutilizable con timeout configurado
    pub http_client: reqwest::Client,
}

impl AppState {
    /// Inicializa un nuevo estado global con el pool de base de datos inyectado.
    #[must_use]
    pub fn new(db_pool: PgPool, tx_ws: broadcast::Sender<TelemetryPayload>) -> Self {
        #[cfg(not(debug_assertions))]
        let jwt_secret = match std::env::var("JWT_SECRET") {
            Ok(s) if s.len() >= 32 => s,
            _ => panic!("JWT_SECRET must be set with at least 32 bytes - refusing to start"),
        };

        #[cfg(debug_assertions)]
        let jwt_secret = match std::env::var("JWT_SECRET") {
            Ok(s) if s.len() >= 32 => s,
            Ok(s) => {
                tracing::warn!(
                    "JWT_SECRET is shorter than 32 bytes. In dev mode, using it anyway with caution."
                );
                s
            }
            Err(_) => {
                tracing::warn!(
                    "JWT_SECRET is not set in dev mode. Generating ephemeral in-memory random secret."
                );
                uuid::Uuid::new_v4().to_string()
            }
        };

        let jwt_encoding_key =
            Arc::new(jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()));
        let jwt_decoding_key =
            Arc::new(jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()));

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client for AppState");

        Self {
            digital_twin: Arc::new(RwLock::new(LruCache::new(TWIN_CAPACITY))),
            db_pool,
            tx_ws,
            pubsub_ready: Arc::new(AtomicBool::new(false)),
            jwt_encoding_key,
            jwt_decoding_key,
            http_client,
        }
    }
}
