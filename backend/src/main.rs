// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.
#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
    unused
)]
#![allow(clippy::needless_for_each)]
// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.
//
// This file is part of the telemetry-hub platform.
// Unauthorized copying of this file, via any medium, is strictly prohibited.
// Proprietary and confidential.
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::multiple_crate_versions
)]

// Modules are now imported from the backend library crate.

// The Askama template has been replaced by the Leptos SPA in `frontend/dist`.
use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        backend::api::digital_twin::get_digital_twin,
        backend::api::chat::chat_with_twin,
        backend::api::dlq::get_recent_dlq,
        backend::api::auth::operator_login
    ),
    components(
        schemas(
            shared::TelemetryPayload,
            shared::Temperature,
            shared::Humidity,
            shared::Ph,
            shared::DissolvedOxygen,
            shared::Pressure,
            shared::GasResistance,
            shared::Co2,
            shared::Iaq,
            shared::Pm1_0,
            shared::Pm2_5,
            shared::Pm10_0,
            shared::GatewayHealthEvent,
            backend::api::chat::ChatRequest,
            backend::api::chat::ChatResponse,
            backend::api::dlq::DlqRecordDto,
            backend::api::auth::LoginRequest,
            backend::api::auth::LoginResponse
        )
    ),tags(
        (name = "telemetry", description = "Endpoints para sensores ambientales"),
        (name = "diagnostics", description = "Eventos de salud del Edge Gateway")
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::Modify;

struct SecurityAddon;
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("Token")
                        .build(),
                ),
            );
        }
    }
}

use axum::http::{header, HeaderName, HeaderValue, Method};
use backend::domain::state::AppState;
use std::sync::Arc;
use std::time::Duration;
use tower::timeout::TimeoutLayer;
use tower_governor::key_extractor::SmartIpKeyExtractor;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;

use axum::{error_handling::HandleErrorLayer, BoxError};

use backend::infrastructure::db::init_db_pool;

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("Received Ctrl-C, shutting down gracefully...");
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Cargar variables de entorno
    let _ = dotenvy::dotenv();

    // Inicializamos el suscriptor de tracing con EnvFilter
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Inicializar Pool de Base de Datos
    let db_pool = init_db_pool().await?;

    // Ejecutar migraciones automáticamente
    tracing::info!("Verificando migraciones SQL...");
    sqlx::migrate!("./migrations").run(&db_pool).await?;

    // Inicializar canales de streaming WebSockets
    let (tx_ws, _rx_ws) = tokio::sync::broadcast::channel(100);

    // Estado concurrente
    let app_state = AppState::new(db_pool.clone(), tx_ws);

    let (tx_shutdown, rx_shutdown) = tokio::sync::broadcast::channel::<()>(1);

    // Iniciar el worker de Pub/Sub en background
    let pubsub_handle = tokio::spawn(backend::infrastructure::pubsub::start_pubsub_worker(
        app_state.clone(),
        rx_shutdown,
    ));

    // Configurar rate limiting para la API pública detrás de proxy/Cloud Run
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(SmartIpKeyExtractor)
            .per_millisecond(200)
            .burst_size(30)
            .finish()
            .ok_or("Invalid governor config")?,
    );

    // Chat API stricter rate limiting: ~5 requests per minute
    let chat_governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(SmartIpKeyExtractor)
            .per_second(12) // 1 request every 12 seconds = 5 req/min
            .burst_size(2)
            .finish()
            .ok_or("Invalid chat governor config")?,
    );

    let cors_layer = if let Ok(origin_str) = std::env::var("CORS_ORIGIN") {
        tracing::info!("CORS restringido a origen: {}", origin_str);
        let origin_val = origin_str.parse::<HeaderValue>()?;
        CorsLayer::new()
            .allow_origin(origin_val)
            .allow_methods([Method::GET, Method::POST])
            .allow_headers([
                header::AUTHORIZATION,
                header::CONTENT_TYPE,
                HeaderName::from_static("x-api-key"),
            ])
    } else {
        tracing::info!(
            "CORS_ORIGIN no configurado: denegando peticiones cross-origin por defecto (Fail-Closed)"
        );
        CorsLayer::new()
    };

    let admin_protected_routes = Router::new()
        .nest("/api/dlq", backend::api::dlq::router())
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            backend::api::auth::auth_middleware,
        ));

    let mut public_app = Router::new()
        .route("/health", axum::routing::get(backend::api::health::liveness_probe))
        .route("/ready", axum::routing::get(backend::api::health::readiness_probe))
        .route("/api/auth/login", axum::routing::post(backend::api::auth::operator_login))
        .route(
            "/api/digital-twin",
            axum::routing::get(backend::api::digital_twin::get_digital_twin),
        )
        .route(
            "/api/chat",
            axum::routing::post(backend::api::chat::chat_with_twin)
                .layer(GovernorLayer { config: chat_governor_conf }),
        )
        .route("/api/ws", axum::routing::get(backend::api::ws::ws_handler))
        .merge(admin_protected_routes);

    if cfg!(debug_assertions) || std::env::var("ENABLE_SWAGGER").as_deref() == Ok("true") {
        tracing::info!("Swagger UI habilitado en /swagger-ui");
        public_app = public_app
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));
    }

    let public_app = public_app
        .fallback_service(
            ServeDir::new("frontend/dist")
                .not_found_service(ServeFile::new("frontend/dist/index.html")),
        )
        .layer(
            tower::ServiceBuilder::new()
                .layer(RequestBodyLimitLayer::new(16 * 1024)) // 16 KB max payload
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    if err.is::<tower::timeout::error::Elapsed>() {
                        (
                            axum::http::StatusCode::REQUEST_TIMEOUT,
                            "Request timed out".to_string(),
                        )
                    } else {
                        (
                            axum::http::StatusCode::TOO_MANY_REQUESTS,
                            "Too many requests or service temporarily unavailable".to_string(),
                        )
                    }
                }))
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                .layer(GovernorLayer { config: governor_conf }),
        )
        .layer(TraceLayer::new_for_http().make_span_with(|req: &axum::extract::Request| {
            tracing::info_span!(
                "http_request",
                method = %req.method(),
                path = %req.uri().path(),
            )
        }))
        .layer(SetResponseHeaderLayer::overriding(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=63072000; includeSubDomains; preload"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data:; connect-src 'self' ws: wss:; frame-ancestors 'none';",
            ),
        ))
        .layer(cors_layer)
        .with_state(app_state.clone());

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let mut host = std::env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    if host == "localhost" {
        host = "127.0.0.1".to_string();
    }

    let public_addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&public_addr).await?;

    tracing::info!("Servidor web público iniciado en http://{}", public_addr);

    // Shutdown gracefully
    let axum_server = axum::serve(
        listener,
        public_app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal());

    axum_server.await?;

    tracing::info!("Servidor web detenido. Notificando a workers para apagar...");
    let _ = tx_shutdown.send(());

    tracing::info!("Esperando a que Pub/Sub Worker termine...");
    let _ = pubsub_handle.await;

    tracing::info!("Cerrando pool de base de datos...");
    app_state.db_pool.close().await;

    tracing::info!("Apagado limpio completado.");

    Ok(())
}
