// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.

use futures::StreamExt;
use gloo_net::websocket::{futures::WebSocket, Message};
use js_sys::Date;
use leptos::*;
use shared::TelemetryPayload;
use std::collections::HashMap;

#[derive(Clone, Debug, serde::Deserialize)]
#[allow(dead_code)]
pub struct DlqRecordDto {
    pub id: String,
    pub ingested_at: String,
    pub raw_payload_base64: String,
    pub error_reason: String,
    pub gateway_id: Option<String>,
    pub event_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DeviceState {
    pub payload: TelemetryPayload,
    pub last_updated_ms: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
}

#[component]
pub fn RoomMonitoring() -> impl IntoView {
    let (twin_state, set_twin_state) = create_signal(HashMap::<String, DeviceState>::new());
    let (conn_status, set_conn_status) = create_signal(ConnectionStatus::Connecting);
    let (current_time, set_current_time) = create_signal(Date::now());
    let (jwt_token, set_jwt_token) = create_signal(None::<String>);
    let (dlq_records, set_dlq_records) = create_signal(Vec::<DlqRecordDto>::new());

    // Timer to update current time for staleness calculation
    create_effect(move |_| {
        let handle = leptos::set_interval_with_handle(
            move || {
                set_current_time.set(Date::now());
            },
            std::time::Duration::from_secs(1),
        );
        on_cleanup(move || {
            if let Ok(h) = handle {
                h.clear();
            }
        });
    });

    // Fetch token on mount
    create_effect(move |_| {
        spawn_local(async move {
            let window = web_sys::window().expect("no global `window` exists");
            let location = window.location();
            let host = location.host().expect("should have a host");
            let protocol = location.protocol().expect("should have a protocol");

            let auth_url = if host.contains("localhost") || host.contains("127.0.0.1") {
                "http://127.0.0.1:3000/api/auth/mock-login".to_string()
            } else {
                format!("{protocol}//{host}/api/auth/mock-login")
            };

            if let Ok(res) = reqwest::Client::new().post(&auth_url).send().await {
                if let Ok(json) = res.json::<serde_json::Value>().await {
                    if let Some(token) = json.get("token").and_then(|t| t.as_str()) {
                        set_jwt_token.set(Some(token.to_string()));
                    }
                }
            }
        });
    });

    // DLQ Polling (every 5 seconds)
    create_effect(move |_| {
        let handle = leptos::set_interval_with_handle(
            move || {
                let Some(token) = jwt_token.get() else {
                    return;
                };

                spawn_local(async move {
                    let window = web_sys::window().expect("no global `window` exists");
                    let location = window.location();
                    let host = location.host().expect("should have a host");
                    let protocol = location.protocol().expect("should have a protocol");

                    let dlq_url = if host.contains("localhost") || host.contains("127.0.0.1") {
                        "http://127.0.0.1:3000/api/dlq/recent".to_string()
                    } else {
                        format!("{protocol}//{host}/api/dlq/recent")
                    };

                    if let Ok(res) = reqwest::Client::new()
                        .get(&dlq_url)
                        .header("Authorization", format!("Bearer {token}"))
                        .send()
                        .await
                    {
                        if let Ok(json) = res.json::<Vec<DlqRecordDto>>().await {
                            set_dlq_records.set(json);
                        }
                    }
                });
            },
            std::time::Duration::from_secs(5),
        );
        on_cleanup(move || {
            if let Ok(h) = handle {
                h.clear();
            }
        });
    });

    let (retry_trigger, set_retry_trigger) = create_signal(false);

    create_effect(move |_| {
        // Depend on retry_trigger to re-run this effect
        retry_trigger.track();

        let Some(token) = jwt_token.get() else {
            return; // Wait until token is available
        };

        let window = web_sys::window().expect("no global `window` exists");
        let location = window.location();
        let host = location.host().expect("should have a host");
        let protocol = location.protocol().expect("should have a protocol");
        let ws_protocol = if protocol == "https:" { "wss:" } else { "ws:" };

        let ws_base = if host.contains("localhost") || host.contains("127.0.0.1") {
            "ws://127.0.0.1:3000/api/ws".to_string()
        } else {
            format!("{ws_protocol}//{host}/api/ws")
        };

        let ws_url = format!("{ws_base}?token={token}");

        set_conn_status.set(ConnectionStatus::Connecting);

        if let Ok(ws) = WebSocket::open(&ws_url) {
            set_conn_status.set(ConnectionStatus::Connected);
            let (_, mut read) = ws.split();

            spawn_local(async move {
                while let Some(msg) = read.next().await {
                    if let Ok(Message::Text(text)) = msg {
                        if let Ok(payload) = serde_json::from_str::<TelemetryPayload>(&text) {
                            set_twin_state.update(|state| {
                                state.insert(
                                    payload.device_id.clone(),
                                    DeviceState {
                                        payload: payload.clone(),
                                        last_updated_ms: Date::now(),
                                    },
                                );
                            });
                        }
                    }
                }
                set_conn_status.set(ConnectionStatus::Disconnected);
                // Trigger reconnect after 3 seconds
                leptos::set_timeout(
                    move || {
                        set_retry_trigger.update(|v| *v = !*v);
                    },
                    std::time::Duration::from_secs(3),
                );
            });
        } else {
            set_conn_status.set(ConnectionStatus::Disconnected);
            leptos::set_timeout(
                move || {
                    set_retry_trigger.update(|v| *v = !*v);
                },
                std::time::Duration::from_secs(3),
            );
        }
    });

    view! {
        <div>
            <h2 class="section-title">"Room Monitoring - HMI Node"</h2>

            <div style="margin-bottom: 2rem;">
                {move || match conn_status.get() {
                    ConnectionStatus::Connected => view! {
                        <div class="status-bar">
                            <span>"● WS CONNECTED"</span>
                            <span>"Live Data Stream Active"</span>
                        </div>
                    }.into_any(),
                    ConnectionStatus::Connecting => view! {
                        <div class="status-bar status-bar-connecting">
                            <span class="pulse-text">"↻ CONNECTING..."</span>
                            <span>"Establishing Link..."</span>
                        </div>
                    }.into_any(),
                    ConnectionStatus::Disconnected => view! {
                        <div class="status-bar status-bar-disconnected">
                            <span>"⨯ DISCONNECTED"</span>
                            <span>"Offline - Waiting for reconnection"</span>
                        </div>
                    }.into_any(),
                }}
            </div>

            <Show
                when=move || !twin_state.get().is_empty()
                fallback=move || view! {
                    <div class="status-waiting">
                        <div class="spinner"></div>
                        <h3 class="pulse-text">"Esperando conexión de Gateway..."</h3>
                        <p>"(GCP Pub/Sub Local Emulator via WebSockets)"</p>
                    </div>
                }
            >
                <div class="project-grid telemetry-grid">
                    {move || twin_state.get().values().cloned().map(|device_state| {
                        let age_ms = current_time.get() - device_state.last_updated_ms;
                        let is_stale = age_ms > 5000.0;
                        let stale_class = if is_stale { "stale-data" } else { "" };
                        let payload = device_state.payload;

                        let latency_ms = (payload.ingested_at - payload.measured_at).num_milliseconds();
                        let latency_display = if latency_ms <= 0 {
                            "< 1 ms".to_string()
                        } else if latency_ms < 1000 {
                            format!("{latency_ms} ms")
                        } else {
                            format!("{:.2} s", latency_ms as f64 / 1000.0)
                        };

                        let is_calibrating = payload.is_calibrating.unwrap_or(false);
                        let is_laser_obstructed = payload.is_laser_obstructed.unwrap_or(false);

                        view! {
                            <section class=format!("terminal-card {}", stale_class)>
                                <div class="card-header">
                                    <span class="dot red" aria-hidden="true"></span>
                                    <span class="dot yellow" aria-hidden="true"></span>
                                    <span class="dot green" aria-hidden="true"></span>
                                    <span class="filename">{payload.device_id.clone()} ".log"</span>
                                    {if is_calibrating {
                                        view! {
                                            <span class="badge-calibrating pulse-text" title="Warmup de sensores BSEC en progreso (12 pulsos iniciales)">
                                                "⏳ Calibrando Sensores"
                                            </span>
                                        }.into_any()
                                    } else {
                                        view! { <span/> }.into_any()
                                    }}
                                </div>
                                <div class="card-body">
                                    {if is_laser_obstructed {
                                        view! {
                                            <div class="alert-laser-obstructed">
                                                <span>"⚠️"</span>
                                                <span><strong>"ALERTA ÓPTICA: "</strong>"Lente Láser Sucia / Obstruida (BMV080)"</span>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <span/> }.into_any()
                                    }}

                                    {payload.temperature.map(|t| view! { <p class="metric-row metric-temp">"Temperature: "<span class="metric-value">{format!("{:.1}", t.value())}</span>" °C"</p> })}
                                    {payload.humidity.map(|h| view! { <p class="metric-row metric-humidity">"Humidity: "<span class="metric-value">{format!("{:.1}", h.value())}</span>" %"</p> })}
                                    {payload.iaq.map(|iaq| view! { <p class="metric-row metric-iaq">"IAQ: "<span class="metric-value">{format!("{:.0}", iaq.value())}</span></p> })}
                                    {payload.co2.map(|c| view! { <p class="metric-row metric-co2">"CO2: "<span class="metric-value">{format!("{:.0}", c.value())}</span>" ppm"</p> })}
                                    {payload.eco2.map(|eco2| view! { <p class="metric-row metric-eco2">"eCO2 (BSEC): "<span class="metric-value">{format!("{:.0}", eco2)}</span>" ppm"</p> })}
                                    {payload.pm2_5.map(|pm| view! { <p class="metric-row metric-pm25">"PM2.5: "<span class="metric-value">{format!("{:.1}", pm.value())}</span>" µg/m³"</p> })}
                                    {payload.pm10_0.map(|pm| view! { <p class="metric-row metric-pm10">"PM10: "<span class="metric-value">{format!("{:.1}", pm.value())}</span>" µg/m³"</p> })}
                                    {payload.pressure.map(|p| view! { <p class="metric-row metric-pressure">"Pressure: "<span class="metric-value">{format!("{:.1}", p.value())}</span>" hPa"</p> })}
                                    {payload.gas_resistance.map(|g| view! { <p class="metric-row metric-gas">"Gas Res: "<span class="metric-value">{g.value()}</span>" Ohms"</p> })}
                                    {payload.ph.map(|ph| view! { <p class="metric-row metric-ph">"pH: "<span class="metric-value">{format!("{:.2}", ph.value())}</span></p> })}
                                    {payload.dissolved_oxygen.map(|do_val| view! { <p class="metric-row metric-do">"DO: "<span class="metric-value">{format!("{:.2}", do_val.value())}</span>" mg/L"</p> })}
                                    {payload.battery_mv.map(|mv| view! { <p class="metric-row metric-battery">"Battery: "<span class="metric-value">{mv}</span>" mV"</p> })}
                                    {payload.sleep_cycles.map(|sc| view! { <p class="metric-row metric-sleep">"Sleep Cycles: "<span class="metric-value">{sc}</span></p> })}

                                    <div class="card-footer-meta">
                                        <div class="latency-meta">
                                            <span class="latency-meta-label">"Latencia Red: "</span>
                                            <span class="latency-badge">{latency_display}</span>
                                        </div>
                                        <div class="data-age">
                                            {if is_stale {
                                                format!("Stale: updated {:.1}s ago", age_ms / 1000.0)
                                            } else {
                                                "Live Data (0s)".to_string()
                                            }}
                                        </div>
                                    </div>
                                </div>
                            </section>
                        }
                    }).collect_view()}
                </div>
            </Show>

            <div class="dlq-section">
                <h3 class="section-title dlq-title">"Terminal de Alertas Críticas (DLQ)"</h3>
                <div class="terminal-card dlq-card">
                    <div class="card-header dlq-header">
                        <span class="dot dlq-dot" aria-hidden="true"></span>
                        <span class="dot dlq-dot" aria-hidden="true"></span>
                        <span class="dot dlq-dot" aria-hidden="true"></span>
                        <span class="filename">"poison_pills.log"</span>
                    </div>
                    <div class="card-body">
                        <Show
                            when=move || !dlq_records.get().is_empty()
                            fallback=move || view! { <p class="dlq-ok-msg">"[OK] No hay Poison Pills recientes en la cola."</p> }
                        >
                            <table class="dlq-table">
                                <thead>
                                    <tr>
                                        <th scope="col">"Timestamp"</th>
                                        <th scope="col">"Gateway"</th>
                                        <th scope="col">"Razón del Rechazo"</th>
                                        <th scope="col">"Payload Base64"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || dlq_records.get().into_iter().map(|record| {
                                        view! {
                                            <tr class="dlq-row">
                                                <td class="dlq-col-time">{record.ingested_at}</td>
                                                <td class="dlq-col-gw">{record.gateway_id.unwrap_or_else(|| "Desconocido".to_string())}</td>
                                                <td class="dlq-col-reason">{record.error_reason}</td>
                                                <td class="dlq-col-payload">
                                                    {record.raw_payload_base64}
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </Show>
                    </div>
                </div>
            </div>
        </div>
    }
}
