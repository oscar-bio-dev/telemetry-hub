// Copyright (c) 2026 Oscar Mora / SetaeSense. All rights reserved.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Utc;
use prost::Message;
use reqwest::Client;
use serde_json::json;
use shared::TelemetryPayloadPb;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let base_topic_url =
        "http://localhost:8085/v1/projects/oscar-bio-dev-project/topics/room-telemetry";
    let topic_url = format!("{base_topic_url}:publish");

    // Ensure topic exists in the emulator
    let _ = client.put(base_topic_url).send().await;

    let mut sequence = 1;
    println!("Starting Dummy Publisher...");
    println!("Sending 21-field Protobuf payloads to {topic_url} every 2 seconds");

    loop {
        let now_ms = u64::try_from(Utc::now().timestamp_millis()).unwrap_or(0);

        let payload = TelemetryPayloadPb {
            protocol_version: 1,
            schema_version: 1,
            event_id: format!("00000000-0000-0000-0000-{sequence:012x}"),
            gateway_id: "Gtw-Lab-01".to_string(),
            device_id: "ESP32-P4-Test".to_string(),
            node_sequence: sequence,
            measured_at_ms: now_ms - 100,
            ingested_at_ms: now_ms,
            temperature: Some(f32::from((sequence % 5) as u16).mul_add(0.1, 24.5)),
            humidity: Some(f32::from((sequence % 10) as u16).mul_add(0.5, 45.0)),
            ph: Some(7.2),
            dissolved_oxygen: Some(8.1),
            pressure: Some(1013.25),
            gas_resistance: Some(12000.0),
            co2: Some(400 + (sequence % 50)),
            iaq: Some(50.0),
            pm1_0: Some(10.0),
            pm2_5: Some(12.5),
            pm10_0: Some(15.0),
            battery_mv: Some(4100),
            sleep_cycles: Some(10),
            is_calibrating: Some(false),
            eco2: Some(450.0),
            bvoc: Some(0.2),
            tvoc: Some(25.0),
            pm1_0_count: Some(100.0),
            pm2_5_count: Some(45.0),
            pm10_0_count: Some(12.0),
            is_laser_obstructed: Some(false),
            is_pm_out_of_range: Some(false),
            laser_runtime: Some(10.5),
            system_error_bitmask: Some(0),
            status: Some(shared::NodeStatus::Monitoring as i32),
        };

        let mut buf = Vec::new();
        payload.encode(&mut buf)?;

        let b64_data = STANDARD.encode(&buf);

        let body = json!({
            "messages": [
                {
                    "data": b64_data
                }
            ]
        });

        match client.post(&topic_url).json(&body).send().await {
            Ok(res) if res.status().is_success() => {
                println!(
                    "Published Event {sequence} -> CO2: {} ppm, Temp: {:.1} °C",
                    payload.co2.unwrap(),
                    payload.temperature.unwrap()
                );
            }
            Ok(res) => {
                println!(
                    "Failed to publish. Status: {}, Body: {:?}",
                    res.status(),
                    res.text().await?
                );
            }
            Err(e) => {
                println!("Connection error: {e}. Is the Pub/Sub emulator running on port 8085?");
            }
        }

        sequence += 1;
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
