// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.

use futures::StreamExt;
use gloo_net::websocket::{futures::WebSocket, Message};
use js_sys::Date;
use leptos::*;
use shared::TelemetryPayload;
use std::collections::{HashMap, VecDeque};

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

/// Estado en memoria por dispositivo incluyendo buffer histórico para análisis de tendencias.
#[derive(Clone, Debug)]
pub struct DeviceState {
    pub payload: TelemetryPayload,
    pub last_updated_ms: f64,
    pub temp_history: VecDeque<f64>,
    pub co2_history: VecDeque<f64>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
}

/// Gráfica de línea de tendencia (Sparkline en SVG de alta resolución) para métricas críticas.
#[component]
pub fn SparklineCanvas(
    data: VecDeque<f64>,
    color: &'static str,
    #[prop(optional)] _unit: Option<&'static str>,
) -> impl IntoView {
    let samples: Vec<f64> = data.into_iter().collect();
    if samples.is_empty() {
        return view! {
            <div class="sparkline-empty">
                <span class="sparkline-placeholder">"Buffer en espera..."</span>
            </div>
        }
        .into_any();
    }

    let mut min_val = f64::INFINITY;
    let mut max_val = f64::NEG_INFINITY;
    for &v in &samples {
        if v < min_val {
            min_val = v;
        }
        if v > max_val {
            max_val = v;
        }
    }

    if (max_val - min_val).abs() < f64::EPSILON {
        min_val -= 0.5;
        max_val += 0.5;
    }

    let range = max_val - min_val;
    let width = 220.0;
    let height = 44.0;
    let pad_y = 6.0;
    let usable_h = height - (pad_y * 2.0);

    let n = samples.len();
    let mut points_str = String::with_capacity(n * 16);
    let mut last_x = 0.0;
    let mut last_y = height / 2.0;

    for (i, &val) in samples.iter().enumerate() {
        let x = if n > 1 { (i as f64 / (n - 1) as f64) * width } else { width / 2.0 };
        let norm_y = (val - min_val) / range;
        let y = height - pad_y - (norm_y * usable_h);

        if i > 0 {
            points_str.push(' ');
        }
        points_str.push_str(&format!("{x:.1},{y:.1}"));

        if i == n - 1 {
            last_x = x;
            last_y = y;
        }
    }

    let area_points = format!("0.0,{height:.1} {points_str} {width:.1},{height:.1}");
    let sample_count = samples.len();

    view! {
        <div class="sparkline-wrapper">
            <div class="sparkline-header">
                <span class="sparkline-range tabular-nums">{format!("Rango: {min_val:.1} - {max_val:.1}")}</span>
                <span class="sparkline-samples tabular-nums">{format!("{sample_count}/60 pts")}</span>
            </div>
            <svg class="sparkline-svg" viewBox="0 0 220 44" preserveAspectRatio="none" aria-label="Línea de tendencia de telemetría">
                <polygon points=area_points class="sparkline-area" style=format!("fill: {color}; opacity: 0.12;") />
                <polyline points=points_str class="sparkline-polyline" style=format!("stroke: {color};") />
                <circle cx=format!("{last_x:.1}") cy=format!("{last_y:.1}") r="3.5" class="sparkline-dot" style=format!("fill: {color};") />
            </svg>
        </div>
    }
    .into_any()
}

/// Componente de lectura de sensor con jerarquía visual industrial:
/// Etiqueta en mayúsculas (0.75rem), valor numérico grande (1.8rem, tabular-nums),
/// unidad normalizada y rango/tolerancia del instrumento físico.
#[component]
pub fn MetricReadout(
    label: &'static str,
    value: String,
    unit: &'static str,
    tolerance: &'static str,
    #[prop(optional)] color_class: Option<&'static str>,
    #[prop(optional)] sparkline: Option<View>,
) -> impl IntoView {
    let color_cls = color_class.unwrap_or("metric-theme-default");
    view! {
        <div class=format!("metric-readout-card {color_cls}")>
            <div class="metric-readout-top">
                <span class="metric-readout-label">{label}</span>
                <span class="metric-readout-tolerance tabular-nums">{tolerance}</span>
            </div>
            <div class="metric-readout-main">
                <span class="metric-readout-value tabular-nums">{value}</span>
                <span class="metric-readout-unit">{unit}</span>
            </div>
            {sparkline.map(|chart| view! {
                <div class="metric-readout-sparkline">
                    {chart}
                </div>
            })}
        </div>
    }
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
        retry_trigger.track();

        let Some(token) = jwt_token.get() else {
            return;
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
                                let (mut temp_history, mut co2_history) = if let Some(existing) =
                                    state.get(&payload.device_id)
                                {
                                    (existing.temp_history.clone(), existing.co2_history.clone())
                                } else {
                                    (VecDeque::with_capacity(60), VecDeque::with_capacity(60))
                                };

                                if let Some(t) = payload.temperature {
                                    temp_history.push_back(t.value());
                                    if temp_history.len() > 60 {
                                        temp_history.pop_front();
                                    }
                                }

                                if let Some(c) = payload.co2 {
                                    co2_history.push_back(c.value());
                                    if co2_history.len() > 60 {
                                        co2_history.pop_front();
                                    }
                                }

                                state.insert(
                                    payload.device_id.clone(),
                                    DeviceState {
                                        payload: payload.clone(),
                                        last_updated_ms: Date::now(),
                                        temp_history,
                                        co2_history,
                                    },
                                );
                            });
                        }
                    }
                }
                set_conn_status.set(ConnectionStatus::Disconnected);
                // Reintento tras 3 segundos
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
        <div class="page-container room-monitoring-page">
            <header class="section-header">
                <h2 class="section-title">"~/telemetry/room - HMI Nodo de Monitoreo en Tiempo Real"</h2>
                <p class="section-desc">
                    "Telemetría continua de sensores ambientales con serialización de alta velocidad, detección de anomalías y búfer de tendencias en memoria."
                </p>
            </header>

            <div class="connection-status-panel">
                {move || match conn_status.get() {
                    ConnectionStatus::Connected => view! {
                        <div class="status-bar status-bar-connected">
                            <span class="status-indicator">"● WS CONNECTED"</span>
                            <span class="status-details">"Canal WebSocket Activo - Transmisión en Tiempo Real"</span>
                        </div>
                    }.into_any(),
                    ConnectionStatus::Connecting => view! {
                        <div class="status-bar status-bar-connecting">
                            <span class="status-indicator pulse-text">"↻ ESTABLISHING LINK..."</span>
                            <span class="status-details">"Negociando handshake de WebSocket..."</span>
                        </div>
                    }.into_any(),
                    ConnectionStatus::Disconnected => view! {
                        <div class="status-bar status-bar-disconnected">
                            <span class="status-indicator">"⨯ DISCONNECTED"</span>
                            <span class="status-details">"Sin conexión - En espera de reconexión automática..."</span>
                        </div>
                    }.into_any(),
                }}
            </div>

            <Show
                when=move || !twin_state.get().is_empty()
                fallback=move || view! {
                    <div class="status-waiting">
                        <div class="spinner"></div>
                        <h3 class="pulse-text">"Esperando telemetría de dispositivos de campo..."</h3>
                        <p>"(Subscriptor GCP Pub/Sub y Gemelo Digital en Memoria)"</p>
                    </div>
                }
            >
                <div class="telemetry-fleet-grid">
                    {move || twin_state.get().values().cloned().map(|device_state| {
                        let age_ms = current_time.get() - device_state.last_updated_ms;
                        let is_stale = age_ms > 5000.0;
                        let stale_class = if is_stale { "stale-data" } else { "" };
                        let payload = device_state.payload;
                        let temp_history = device_state.temp_history;
                        let co2_history = device_state.co2_history;

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
                            <section class=format!("terminal-card node-card {}", stale_class)>
                                <div class="card-header node-header">
                                    <span class="dot red" aria-hidden="true"></span>
                                    <span class="dot yellow" aria-hidden="true"></span>
                                    <span class="dot green" aria-hidden="true"></span>
                                    <span class="filename">
                                        <span class="device-tag">"NODE_ID::"</span>
                                        <strong class="device-id">{payload.device_id.clone()}</strong>
                                        ".log"
                                    </span>
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
                                <div class="card-body node-body">
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

                                    <div class="readout-grid">
                                        {payload.temperature.map(|t| view! {
                                            <MetricReadout
                                                label="Temperatura"
                                                value=format!("{:.1}", t.value())
                                                unit="°C"
                                                tolerance="BME688 ±0.5 °C"
                                                color_class="metric-temp"
                                                sparkline=view! {
                                                    <SparklineCanvas
                                                        data=temp_history.clone()
                                                        color="var(--accent-yellow)"
                                                    />
                                                }.into_view()
                                            />
                                        })}

                                        {payload.co2.map(|c| view! {
                                            <MetricReadout
                                                label="Dióxido de Carbono"
                                                value=format!("{:.0}", c.value())
                                                unit="ppm"
                                                tolerance="SCD41 ±(40 ppm + 5%)"
                                                color_class="metric-co2"
                                                sparkline=view! {
                                                    <SparklineCanvas
                                                        data=co2_history.clone()
                                                        color="var(--accent-red)"
                                                    />
                                                }.into_view()
                                            />
                                        })}

                                        {payload.humidity.map(|h| view! {
                                            <MetricReadout
                                                label="Humedad Relativa"
                                                value=format!("{:.1}", h.value())
                                                unit="%RH"
                                                tolerance="BME688 ±3 %RH"
                                                color_class="metric-humidity"
                                            />
                                        })}

                                        {payload.pm2_5.map(|pm| view! {
                                            <MetricReadout
                                                label="Particulado PM2.5"
                                                value=format!("{:.1}", pm.value())
                                                unit="µg/m³"
                                                tolerance="BMV080 ±10%"
                                                color_class="metric-pm25"
                                            />
                                        })}

                                        {payload.pm10_0.map(|pm| view! {
                                            <MetricReadout
                                                label="Particulado PM10"
                                                value=format!("{:.1}", pm.value())
                                                unit="µg/m³"
                                                tolerance="BMV080 ±15%"
                                                color_class="metric-pm10"
                                            />
                                        })}

                                        {payload.iaq.map(|iaq| view! {
                                            <MetricReadout
                                                label="Índice IAQ"
                                                value=format!("{:.0}", iaq.value())
                                                unit="IAQ"
                                                tolerance="BSEC 0-500"
                                                color_class="metric-iaq"
                                            />
                                        })}

                                        {payload.eco2.map(|eco2| view! {
                                            <MetricReadout
                                                label="eCO2 Estimado"
                                                value=format!("{:.0}", eco2)
                                                unit="ppm"
                                                tolerance="BSEC Estimado"
                                                color_class="metric-eco2"
                                            />
                                        })}

                                        {payload.pressure.map(|p| view! {
                                            <MetricReadout
                                                label="Presión Barométrica"
                                                value=format!("{:.1}", p.value())
                                                unit="hPa"
                                                tolerance="BME688 ±0.6 hPa"
                                                color_class="metric-pressure"
                                            />
                                        })}

                                        {payload.gas_resistance.map(|g| view! {
                                            <MetricReadout
                                                label="Resistencia de Gas"
                                                value=format!("{:.0}", g.value())
                                                unit="Ω"
                                                tolerance="BME688 MOX"
                                                color_class="metric-gas"
                                            />
                                        })}

                                        {payload.ph.map(|ph| view! {
                                            <MetricReadout
                                                label="pH del Medio"
                                                value=format!("{:.2}", ph.value())
                                                unit="pH"
                                                tolerance="AFE ±0.02 pH"
                                                color_class="metric-ph"
                                            />
                                        })}

                                        {payload.dissolved_oxygen.map(|do_val| view! {
                                            <MetricReadout
                                                label="Oxígeno Disuelto"
                                                value=format!("{:.2}", do_val.value())
                                                unit="mg/L"
                                                tolerance="WQM-V100 ±0.1 mg/L"
                                                color_class="metric-do"
                                            />
                                        })}

                                        {payload.battery_mv.map(|mv| view! {
                                            <MetricReadout
                                                label="Voltaje Batería"
                                                value=format!("{mv}")
                                                unit="mV"
                                                tolerance="LiFePO4 Nom. 3.2V"
                                                color_class="metric-battery"
                                            />
                                        })}

                                        {payload.sleep_cycles.map(|sc| view! {
                                            <MetricReadout
                                                label="Ciclos de Sleep"
                                                value=format!("{sc}")
                                                unit="ciclos"
                                                tolerance="Deep-Sleep Counter"
                                                color_class="metric-sleep"
                                            />
                                        })}
                                    </div>

                                    <div class="card-footer-meta">
                                        <div class="latency-meta">
                                            <span class="latency-meta-label">"Latencia Red: "</span>
                                            <span class="latency-badge">{latency_display}</span>
                                        </div>
                                        <div class="data-age">
                                            {if is_stale {
                                                format!("Stale: actualizado hace {:.1}s", age_ms / 1000.0)
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
                            <div class="spec-table-wrapper">
                                <table class="dlq-table tabular-nums">
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
                            </div>
                        </Show>
                    </div>
                </div>
            </div>
        </div>
    }
}
