// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.
// Proprietary and confidential.

//! Background worker (Subscriber) for GCP Pub/Sub.

use crate::domain::state::AppState;
use google_cloud_pubsub::client::Subscriber;
use prost::Message;
use shared::{TelemetryPayload, TelemetryPayloadPb};
use validator::Validate;

/// Starts the Pub/Sub subscriber in the background con exponential backoff and graceful shutdown.
#[allow(clippy::too_many_lines)]
pub async fn start_pubsub_worker(
    state: AppState,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) {
    let mut backoff = std::time::Duration::from_secs(1);
    let max_backoff = std::time::Duration::from_secs(60);

    loop {
        tracing::info!("Intentando conectar a Pub/Sub...");

        match try_init_pubsub().await {
            Ok((client, subscription_name)) => {
                tracing::info!("Conectado a Pub/Sub en {}. Iniciando stream...", subscription_name);
                state.pubsub_ready.store(true, std::sync::atomic::Ordering::SeqCst);

                backoff = std::time::Duration::from_secs(1);

                let mut stream = client.subscribe(&subscription_name).build();

                loop {
                    tokio::select! {
                        _ = shutdown_rx.recv() => {
                            tracing::info!("Señal de apagado recibida en Pub/Sub Worker. Saliendo...");
                            state.pubsub_ready.store(false, std::sync::atomic::Ordering::SeqCst);
                            return;
                        }
                        res = stream.next() => {
                            match res {
                                Some(Ok((message, handler))) => {
                                    let msg_id = message.message_id.clone();
                                    tracing::debug!("Recibido mensaje de Pub/Sub (ID: {})", msg_id);

                                    let pb_dto = match TelemetryPayloadPb::decode(message.data.as_ref()) {
                                        Ok(dto) => dto,
                                        Err(e) => {
                                            tracing::error!("Poison Pill (Protobuf inválido). Msg ID: {}, Error: {}", msg_id, e);
                                            handle_poison_pill(&state, message.data.to_vec(), "Invalid Protobuf", None).await;
                                            handler.ack();
                                            continue;
                                        }
                                    };

                                    let payload: TelemetryPayload = match pb_dto.try_into() {
                                        Ok(p) => p,
                                        Err(e) => {
                                            tracing::error!("Poison Pill (Dominio inválido). Msg ID: {}, Error: {}", msg_id, e);
                                            handle_poison_pill(&state, message.data.to_vec(), &format!("Domain validation failed: {e}"), None).await;
                                            handler.ack();
                                            continue;
                                        }
                                    };

                                    if let Err(e) = payload.validate() {
                                        tracing::error!("Poison Pill (Validación estricta fallida). Msg ID: {}, Error: {}", msg_id, e);
                                        handle_poison_pill(&state, message.data.to_vec(), &format!("Strict validation failed: {e}"), Some(&payload.event_id)).await;
                                        handler.ack();
                                        continue;
                                    }

                                    let event_id_uuid = match uuid::Uuid::parse_str(&payload.event_id) {
                                        Ok(uuid) => uuid,
                                        Err(e) => {
                                            tracing::error!(
                                                "Poison Pill (event_id no es un UUID válido). Msg ID: {}, event_id: {}, Error: {}",
                                                msg_id,
                                                payload.event_id,
                                                e
                                            );
                                            handle_poison_pill(
                                                &state,
                                                message.data.to_vec(),
                                                &format!("Invalid UUID in event_id: {e}"),
                                                None,
                                            )
                                            .await;
                                            handler.ack();
                                            continue;
                                        }
                                    };

                                    let event_id = payload.event_id.clone();

                                    {
                                        let mut twin = state.digital_twin.write().await;
                                        twin.put(payload.device_id.clone(), payload.clone());
                                    }

                                    let _ = state.tx_ws.send(payload.clone());

                                    match persist_telemetry(&state, &payload, event_id_uuid).await {
                                        Ok(()) => {
                                            tracing::info!("Mensaje {} persistido exitosamente (ACK)", msg_id);
                                            handler.ack();
                                        }
                                        Err(e) => {
                                            tracing::error!("Fallo de Infraestructura (TimescaleDB). Haciendo NACK del mensaje {} (Event: {}). Error: {}", msg_id, event_id, e);
                                            handler.nack();
                                        }
                                    }
                                }
                                Some(Err(e)) => {
                                    tracing::error!("Fallo en stream de Pub/Sub: {}", e);
                                    break;
                                }
                                None => {
                                    tracing::warn!("Stream de Pub/Sub cerrado por el servidor.");
                                    break;
                                }
                            }
                        }
                    }
                }

                state.pubsub_ready.store(false, std::sync::atomic::Ordering::SeqCst);
            }
            Err(e) => {
                tracing::error!(
                    "Fallo al inicializar Pub/Sub: {}. Reintentando en {:?}",
                    e,
                    backoff
                );
            }
        }

        tokio::select! {
            _ = shutdown_rx.recv() => {
                tracing::info!("Señal de apagado recibida durante backoff. Saliendo...");
                return;
            }
            () = tokio::time::sleep(backoff) => {
                backoff = std::cmp::min(backoff * 2, max_backoff);
            }
        }
    }
}

async fn try_init_pubsub() -> Result<
    (google_cloud_pubsub::client::Subscriber, String),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let mut builder = Subscriber::builder();
    if let Ok(emulator_host) = std::env::var("PUBSUB_EMULATOR_HOST") {
        let endpoint = format!("http://{emulator_host}");
        let anon = google_cloud_auth::credentials::anonymous::Builder::new().build();
        builder = builder.with_endpoint(endpoint).with_credentials(anon);
    }
    let client = builder.build().await?;

    let subscription_name = std::env::var("PUBSUB_SUBSCRIPTION_NAME").unwrap_or_else(|_| {
        "projects/oscar-bio-dev-project/subscriptions/room-telemetry-sub".to_string()
    });

    if let Ok(emulator_host) = std::env::var("PUBSUB_EMULATOR_HOST") {
        let http = reqwest::Client::new();
        let topic_name = "projects/oscar-bio-dev-project/topics/room-telemetry";

        let topic_url = format!("http://{emulator_host}/v1/{topic_name}");
        let _ = http.put(&topic_url).send().await;

        let sub_url = format!("http://{emulator_host}/v1/{subscription_name}");
        let payload = serde_json::json!({ "topic": topic_name });
        let _ = http.put(&sub_url).json(&payload).send().await;
    }

    Ok((client, subscription_name))
}

/// Inserta la telemetría en `TimescaleDB`. Retorna error si la DB falla.
async fn persist_telemetry(
    state: &AppState,
    payload: &TelemetryPayload,
    event_id: uuid::Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        INSERT INTO telemetry (
            event_id, protocol_version, schema_version, gateway_id, device_id, node_sequence,
            measured_at, ingested_at, temperature, humidity, ph, dissolved_oxygen,
            pressure, gas_resistance, co2, iaq, pm1_0, pm2_5, pm10_0,
            battery_mv, sleep_cycles,
            is_calibrating, eco2, bvoc, tvoc, pm1_0_count, pm2_5_count, pm10_0_count,
            is_laser_obstructed, is_pm_out_of_range, laser_runtime, system_error_bitmask, status
        )
        VALUES (
            $1, $2, $3, $4, $5, $6,
            $7, $8, $9, $10, $11, $12,
            $13, $14, $15, $16, $17, $18, $19,
            $20, $21,
            $22, $23, $24, $25, $26, $27, $28,
            $29, $30, $31, $32, $33
        )
        ON CONFLICT (event_id, measured_at) DO NOTHING
        ",
    )
    .bind(event_id)
    .bind(i32::try_from(payload.protocol_version).unwrap_or(1))
    .bind(i32::try_from(payload.schema_version).unwrap_or(1))
    .bind(&payload.gateway_id)
    .bind(&payload.device_id)
    .bind(i32::try_from(payload.node_sequence).unwrap_or(0))
    .bind(payload.measured_at)
    .bind(payload.ingested_at)
    .bind(payload.temperature.map(shared::Temperature::value))
    .bind(payload.humidity.map(shared::Humidity::value))
    .bind(payload.ph.map(shared::Ph::value))
    .bind(payload.dissolved_oxygen.map(shared::DissolvedOxygen::value))
    .bind(payload.pressure.map(shared::Pressure::value))
    .bind(payload.gas_resistance.map(shared::GasResistance::value))
    .bind(payload.co2.map(shared::Co2::value))
    .bind(payload.iaq.map(shared::Iaq::value))
    .bind(payload.pm1_0.map(shared::Pm1_0::value))
    .bind(payload.pm2_5.map(shared::Pm2_5::value))
    .bind(payload.pm10_0.map(shared::Pm10_0::value))
    .bind(payload.battery_mv.map(|v| i32::try_from(v).unwrap_or(i32::MAX)))
    .bind(payload.sleep_cycles.map(|v| i32::try_from(v).unwrap_or(i32::MAX)))
    .bind(payload.is_calibrating)
    .bind(payload.eco2)
    .bind(payload.bvoc)
    .bind(payload.tvoc)
    .bind(payload.pm1_0_count)
    .bind(payload.pm2_5_count)
    .bind(payload.pm10_0_count)
    .bind(payload.is_laser_obstructed)
    .bind(payload.is_pm_out_of_range)
    .bind(payload.laser_runtime)
    .bind(payload.system_error_bitmask.map(i64::from))
    .bind(payload.status.map(shared::NodeStatus::as_str))
    .execute(&state.db_pool)
    .await?;

    Ok(())
}

const MAX_DLQ_PAYLOAD_BYTES: usize = 4096;

/// Guarda un payload inválido (Poison Pill) en la tabla DLQ.
/// Si falla la inserción a DLQ, solo registramos error en log para no saturar Pub/Sub con NACKs por mensajes intrínsecamente malos.
async fn handle_poison_pill(
    state: &AppState,
    raw_payload: Vec<u8>,
    error_reason: &str,
    event_id: Option<&str>,
) {
    let parsed_event_id = event_id.and_then(|id| uuid::Uuid::parse_str(id).ok());
    let truncated_payload = if raw_payload.len() > MAX_DLQ_PAYLOAD_BYTES {
        raw_payload[..MAX_DLQ_PAYLOAD_BYTES].to_vec()
    } else {
        raw_payload
    };

    let result = sqlx::query(
        r"
        INSERT INTO telemetry_dlq (raw_payload, error_reason, event_id)
        VALUES ($1, $2, $3)
        ",
    )
    .bind(truncated_payload)
    .bind(error_reason)
    .bind(parsed_event_id)
    .execute(&state.db_pool)
    .await;

    if let Err(e) = result {
        tracing::error!(
            "CRITICAL: Falló la escritura en DLQ PostgreSQL. Poison Pill perdida. Error: {}",
            e
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_valid_uuid_event_id() {
        let valid_str = "a1b2c3d4-e5f6-7a8b-9c0d-1e2f3a4b5c6d";
        let parsed = Uuid::parse_str(valid_str);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_invalid_uuid_event_id_detected() {
        let invalid_str = "evt-random-string-123";
        let parsed = Uuid::parse_str(invalid_str);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_payload_truncation_constant() {
        assert_eq!(MAX_DLQ_PAYLOAD_BYTES, 4096);
        let oversized = vec![0u8; 8192];
        let truncated = if oversized.len() > MAX_DLQ_PAYLOAD_BYTES {
            oversized[..MAX_DLQ_PAYLOAD_BYTES].to_vec()
        } else {
            oversized
        };
        assert_eq!(truncated.len(), 4096);
    }
}
