// Copyright (c) 2026 Oscar Mora / SetaeSense. All rights reserved.

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
                        <div class="status-bar" style="background-color: var(--accent-green); color: var(--bg-hard);">
                            <span>"● WS CONNECTED"</span>
                            <span>"Live Data Stream Active"</span>
                        </div>
                    }.into_any(),
                    ConnectionStatus::Connecting => view! {
                        <div class="status-bar" style="background-color: var(--accent-yellow); color: var(--bg-hard);">
                            <span class="pulse-text">"↻ CONNECTING..."</span>
                            <span>"Establishing Link..."</span>
                        </div>
                    }.into_any(),
                    ConnectionStatus::Disconnected => view! {
                        <div class="status-bar" style="background-color: var(--accent-red); color: var(--fg-main);">
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
                        <p style="color: var(--fg-muted);">"(GCP Pub/Sub Local Emulator via WebSockets)"</p>
                    </div>
                }
            >
                <div class="project-grid" style="grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); display: grid;">
                    {move || twin_state.get().values().cloned().map(|device_state| {
                        let age_ms = current_time.get() - device_state.last_updated_ms;
                        let is_stale = age_ms > 5000.0;
                        let stale_class = if is_stale { "stale-data" } else { "" };
                        let payload = device_state.payload;

                        view! {
                            <section class=format!("terminal-card {}", stale_class)>
                                <div class="card-header">
                                    <span class="dot red" aria-hidden="true"></span>
                                    <span class="dot yellow" aria-hidden="true"></span>
                                    <span class="dot green" aria-hidden="true"></span>
                                    <span class="filename">{payload.device_id.clone()} ".log"</span>
                                </div>
                                <div class="card-body">
                                    {payload.temperature.map(|t| view! { <p style="color: var(--accent-yellow);">"Temperature: " {format!("{:.1}", t.value())} " °C"</p> })}
                                    {payload.humidity.map(|h| view! { <p style="color: var(--accent-blue);">"Humidity: " {format!("{:.1}", h.value())} " %"</p> })}
                                    {payload.iaq.map(|iaq| view! { <p style="color: var(--accent-green);">"IAQ: " {format!("{:.0}", iaq.value())}</p> })}
                                    {payload.co2.map(|c| view! { <p style="color: var(--accent-red);">"CO2: " {format!("{:.0}", c.value())} " ppm"</p> })}
                                    {payload.pm2_5.map(|pm| view! { <p style="color: var(--accent-yellow);">"PM2.5: " {format!("{:.1}", pm.value())} " µg/m³"</p> })}
                                    {payload.pm10_0.map(|pm| view! { <p style="color: var(--fg-main);">"PM10: " {format!("{:.1}", pm.value())} " µg/m³"</p> })}
                                    {payload.pressure.map(|p| view! { <p style="color: var(--accent-yellow);">"Pressure: " {format!("{:.1}", p.value())} " hPa"</p> })}
                                    {payload.gas_resistance.map(|g| view! { <p style="color: var(--accent-blue);">"Gas Res: " {g.value()} " Ohms"</p> })}
                                    {payload.ph.map(|ph| view! { <p style="color: var(--accent-green);">"pH: " {format!("{:.2}", ph.value())}</p> })}
                                    {payload.dissolved_oxygen.map(|do_val| view! { <p style="color: var(--fg-main);">"DO: " {format!("{:.2}", do_val.value())} " mg/L"</p> })}
                                    {payload.battery_mv.map(|mv| view! { <p style="color: var(--accent-blue);">"Battery: " {mv} " mV"</p> })}
                                    {payload.sleep_cycles.map(|sc| view! { <p style="color: var(--fg-dim);">"Sleep Cycles: " {sc}</p> })}

                                    <div class="data-age">
                                        {if is_stale {
                                            format!("Stale: updated {:.1}s ago", age_ms / 1000.0)
                                        } else {
                                            "Live Data (0s)".to_string()
                                        }}
                                    </div>
                                </div>
                            </section>
                        }
                    }).collect_view()}
                </div>
            </Show>

            <div style="margin-top: 3rem;">
                <h3 class="section-title" style="color: var(--accent-red);">"Terminal de Alertas Críticas (DLQ)"</h3>
                <div class="terminal-card" style="max-height: 400px; overflow-y: auto;">
                    <div class="card-header" style="background-color: var(--accent-red); color: var(--bg-hard);">
                        <span class="dot" style="background-color: var(--bg-hard);" aria-hidden="true"></span>
                        <span class="dot" style="background-color: var(--bg-hard);" aria-hidden="true"></span>
                        <span class="dot" style="background-color: var(--bg-hard);" aria-hidden="true"></span>
                        <span class="filename">"poison_pills.log"</span>
                    </div>
                    <div class="card-body">
                        <Show
                            when=move || !dlq_records.get().is_empty()
                            fallback=move || view! { <p style="color: var(--accent-green);">"[OK] No hay Poison Pills recientes en la cola."</p> }
                        >
                            <table style="width: 100%; border-collapse: collapse; font-size: 0.9em;">
                                <thead>
                                    <tr style="border-bottom: 1px solid var(--border-color); color: var(--fg-muted);">
                                        <th style="text-align: left; padding: 0.5rem;">"Timestamp"</th>
                                        <th style="text-align: left; padding: 0.5rem;">"Gateway"</th>
                                        <th style="text-align: left; padding: 0.5rem;">"Razón del Rechazo"</th>
                                        <th style="text-align: left; padding: 0.5rem;">"Payload Base64"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || dlq_records.get().into_iter().map(|record| {
                                        view! {
                                            <tr style="border-bottom: 1px dashed var(--bg-soft);">
                                                <td style="padding: 0.5rem; color: var(--fg-dim);">{record.ingested_at}</td>
                                                <td style="padding: 0.5rem; color: var(--accent-blue);">{record.gateway_id.unwrap_or_else(|| "Desconocido".to_string())}</td>
                                                <td style="padding: 0.5rem; color: var(--accent-yellow); font-weight: bold;">{record.error_reason}</td>
                                                <td style="padding: 0.5rem; color: var(--fg-muted); font-family: monospace; font-size: 0.8em; word-break: break-all; max-width: 200px;">
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
