// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.

use crate::types::{ConnectionStatus, DlqFetchState};
use crate::utils::{api_url, ws_url};
use futures::StreamExt;
use gloo_net::websocket::{futures::WebSocket, Message};
use js_sys::Date;
use leptos::*;
use shared::{NodeStatus, TelemetryPayload};
use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::fmt::Write;
use std::rc::Rc;

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
#[derive(Clone, Debug, PartialEq)]
pub struct DeviceState {
    pub payload: TelemetryPayload,
    pub last_updated_ms: f64,
    pub temp_history: VecDeque<f64>,
    pub co2_history: VecDeque<f64>,
}

/// Gráfica de línea de tendencia (Sparkline en SVG de alta resolución) para métricas críticas.
#[component]
#[allow(clippy::cast_precision_loss)]
pub fn SparklineCanvas(
    data: VecDeque<f64>,
    color: &'static str,
    #[prop(optional)] unit: Option<&'static str>,
) -> impl IntoView {
    let _ = unit;
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
        let y = norm_y.mul_add(-usable_h, height - pad_y);

        if i > 0 {
            points_str.push(' ');
        }
        let _ = write!(points_str, "{x:.1},{y:.1}");

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

/// Tarjeta de telemetría por nodo con reactividad granular desacoplada del timer global.
#[component]
#[allow(clippy::too_many_lines)]
fn NodeCard(
    device_id: String,
    twin_state: ReadSignal<HashMap<String, DeviceState>>,
    current_time: ReadSignal<f64>,
) -> impl IntoView {
    let id_for_memo = device_id.clone();
    let device_state_memo =
        create_memo(move |_| twin_state.with(|map| map.get(&id_for_memo).cloned()));

    let payload =
        create_memo(move |_| device_state_memo.with(|s| s.as_ref().map(|d| d.payload.clone())));

    let last_updated = create_memo(move |_| {
        device_state_memo.with(|state| state.as_ref().map_or(0.0, |s| s.last_updated_ms))
    });

    // Reactividad granular: sólo is_stale y age_text dependen de current_time.
    // El tick de 1 segundo actualiza únicamente la clase CSS y el texto de edad.
    let is_stale = create_memo(move |_| {
        let now = current_time.get();
        let updated = last_updated.get();
        (now - updated) > 5000.0
    });

    let age_text = create_memo(move |_| {
        let now = current_time.get();
        let updated = last_updated.get();
        #[allow(clippy::cast_precision_loss)]
        let diff_s = (now - updated) / 1000.0;
        if diff_s > 5.0 {
            format!("Stale: actualizado hace {diff_s:.1}s")
        } else {
            "Live Data (0s)".to_string()
        }
    });

    let latency_display = create_memo(move |_| {
        payload.with(|p| {
            let Some(p) = p else {
                return "< 1 ms".to_string();
            };
            let latency_ms = (p.ingested_at - p.measured_at).num_milliseconds();
            if latency_ms <= 0 {
                "< 1 ms".to_string()
            } else if latency_ms < 1000 {
                format!("{latency_ms} ms")
            } else {
                #[allow(clippy::cast_precision_loss)]
                let seconds = latency_ms as f64 / 1000.0;
                format!("{seconds:.2} s")
            }
        })
    });

    let temp_history = create_memo(move |_| {
        device_state_memo
            .with(|s| s.as_ref().map_or_else(VecDeque::new, |d| d.temp_history.clone()))
    });

    let co2_history = create_memo(move |_| {
        device_state_memo.with(|s| s.as_ref().map_or_else(VecDeque::new, |d| d.co2_history.clone()))
    });

    let status_info = create_memo(move |_| {
        payload.with(|p| {
            let status = p.as_ref().and_then(|d| d.status).unwrap_or(NodeStatus::Monitoring);
            let (status_class, led_class) = match status {
                NodeStatus::Monitoring => ("status-ok", "node-status-led led-ok"),
                NodeStatus::Calibrating | NodeStatus::SelfTesting => {
                    ("status-warn", "node-status-led led-warn")
                }
                NodeStatus::HardwareError => {
                    ("status-fault", "node-status-led led-fault blink-fast")
                }
            };
            (status, status_class, led_class)
        })
    });

    view! {
        <section
            class="terminal-card node-card"
            class:stale-data=move || is_stale.get()
        >
            <div class="card-header node-header">
                <div class="node-status-container">
                    <span class=move || status_info.get().2 aria-hidden="true"></span>
                    <span class=move || format!("node-status-text {}", status_info.get().1)>
                        {move || status_info.get().0.as_str()}
                    </span>
                </div>
                <span class="filename">
                    <span class="device-tag">"NODE_ID::"</span>
                    <strong class="device-id">{device_id}</strong>
                    ".log"
                </span>
                {move || {
                    let is_calibrating = payload.with(|p| p.as_ref().and_then(|d| d.is_calibrating).unwrap_or(false));
                    if is_calibrating {
                        view! {
                            <span class="badge-calibrating pulse-text" title="Warmup de sensores BSEC en progreso (12 pulsos iniciales)">
                                <svg class="badge-svg-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                                    <circle cx="12" cy="12" r="10"></circle>
                                    <polyline points="12 6 12 12 16 14"></polyline>
                                </svg>
                                <span>"Calibrando Sensores"</span>
                            </span>
                        }.into_any()
                    } else {
                        view! { <span/> }.into_any()
                    }
                }}
            </div>
            <div class="card-body node-body">
                {move || {
                    let (error_bitmask, pm_out_of_range) = payload.with(|p| {
                        p.as_ref().map_or((0u32, false), |d| {
                            (
                                d.system_error_bitmask.unwrap_or(0),
                                d.is_pm_out_of_range.unwrap_or(false),
                            )
                        })
                    });

                    if error_bitmask != 0 || pm_out_of_range {
                        view! {
                            <div class="alert-hardware-fault" role="alert">
                                <svg class="alert-svg-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                                    <circle cx="12" cy="12" r="10"></circle>
                                    <line x1="12" y1="8" x2="12" y2="12"></line>
                                    <line x1="12" y1="16" x2="12.01" y2="16"></line>
                                </svg>
                                <div class="alert-hardware-text">
                                    <strong>"FALLO DE HARDWARE: "</strong>
                                    {if error_bitmask != 0 {
                                        format!("Bitmask 0x{error_bitmask:04X} ")
                                    } else {
                                        String::new()
                                    }}
                                    {if pm_out_of_range {
                                        "[SATURACIÓN BMV080 PM]"
                                    } else {
                                        ""
                                    }}
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! { <span/> }.into_any()
                    }
                }}

                {move || {
                    let is_laser_obstructed = payload.with(|p| p.as_ref().and_then(|d| d.is_laser_obstructed).unwrap_or(false));
                    if is_laser_obstructed {
                        view! {
                            <div class="alert-laser-obstructed" role="alert">
                                <svg class="alert-svg-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                                    <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path>
                                    <line x1="12" y1="9" x2="12" y2="13"></line>
                                    <line x1="12" y1="17" x2="12.01" y2="17"></line>
                                </svg>
                                <span><strong>"ALERTA ÓPTICA: "</strong>"Lente Láser Sucia / Obstruida (BMV080)"</span>
                            </div>
                        }.into_any()
                    } else {
                        view! { <span/> }.into_any()
                    }
                }}

                {move || payload.get().map(|p| {
                    let th = temp_history.get();
                    let ch = co2_history.get();
                    view! {
                        <div class="readout-grid">
                            {p.temperature.map(|t| view! {
                                <MetricReadout
                                    label="Temperatura"
                                    value=format!("{:.1}", t.value())
                                    unit="°C"
                                    tolerance="BME688 ±0.5 °C"
                                    color_class="metric-temp"
                                    sparkline=view! {
                                        <SparklineCanvas
                                            data=th.clone()
                                            color="var(--accent-yellow)"
                                        />
                                    }.into_view()
                                />
                            })}

                            {p.co2.map(|c| view! {
                                <MetricReadout
                                    label="Dióxido de Carbono"
                                    value=format!("{:.0}", c.value())
                                    unit="ppm"
                                    tolerance="SCD41 ±(40 ppm + 5%)"
                                    color_class="metric-co2"
                                    sparkline=view! {
                                        <SparklineCanvas
                                            data=ch.clone()
                                            color="var(--accent-red)"
                                        />
                                    }.into_view()
                                />
                            })}

                            {p.humidity.map(|h| view! {
                                <MetricReadout
                                    label="Humedad Relativa"
                                    value=format!("{:.1}", h.value())
                                    unit="%RH"
                                    tolerance="BME688 ±3 %RH"
                                    color_class="metric-humidity"
                                />
                            })}

                            {p.pm2_5.map(|pm| view! {
                                <MetricReadout
                                    label="Particulado PM2.5"
                                    value=format!("{:.1}", pm.value())
                                    unit="µg/m³"
                                    tolerance="BMV080 ±10%"
                                    color_class="metric-pm25"
                                />
                            })}

                            {p.pm10_0.map(|pm| view! {
                                <MetricReadout
                                    label="Particulado PM10"
                                    value=format!("{:.1}", pm.value())
                                    unit="µg/m³"
                                    tolerance="BMV080 ±15%"
                                    color_class="metric-pm10"
                                />
                            })}

                            {p.iaq.map(|iaq| view! {
                                <MetricReadout
                                    label="Índice IAQ"
                                    value=format!("{:.0}", iaq.value())
                                    unit="IAQ"
                                    tolerance="BSEC 0-500"
                                    color_class="metric-iaq"
                                />
                            })}

                            {p.eco2.map(|eco2| view! {
                                <MetricReadout
                                    label="eCO2 Estimado"
                                    value=format!("{:.0}", eco2)
                                    unit="ppm"
                                    tolerance="BSEC Estimado"
                                    color_class="metric-eco2"
                                />
                            })}

                            {p.pressure.map(|p_val| view! {
                                <MetricReadout
                                    label="Presión Barométrica"
                                    value=format!("{:.1}", p_val.value())
                                    unit="hPa"
                                    tolerance="BME688 ±0.6 hPa"
                                    color_class="metric-pressure"
                                />
                            })}

                            {p.gas_resistance.map(|g| view! {
                                <MetricReadout
                                    label="Resistencia de Gas"
                                    value=format!("{:.0}", g.value())
                                    unit="Ω"
                                    tolerance="BME688 MOX"
                                    color_class="metric-gas"
                                />
                            })}

                            {p.ph.map(|ph_val| view! {
                                <MetricReadout
                                    label="pH del Medio"
                                    value=format!("{:.2}", ph_val.value())
                                    unit="pH"
                                    tolerance="AFE ±0.02 pH"
                                    color_class="metric-ph"
                                />
                            })}

                            {p.dissolved_oxygen.map(|do_val| view! {
                                <MetricReadout
                                    label="Oxígeno Disuelto"
                                    value=format!("{:.2}", do_val.value())
                                    unit="mg/L"
                                    tolerance="WQM-V100 ±0.1 mg/L"
                                    color_class="metric-do"
                                />
                            })}

                            {p.battery_mv.map(|mv| view! {
                                <MetricReadout
                                    label="Voltaje Batería"
                                    value=format!("{mv}")
                                    unit="mV"
                                    tolerance="LiFePO4 Nom. 3.2V"
                                    color_class="metric-battery"
                                />
                            })}

                            {p.sleep_cycles.map(|sc| view! {
                                <MetricReadout
                                    label="Ciclos de Sleep"
                                    value=format!("{sc}")
                                    unit="ciclos"
                                    tolerance="Deep-Sleep Counter"
                                    color_class="metric-sleep"
                                />
                            })}
                        </div>
                    }
                })}

                <div class="card-footer-meta">
                    <div class="latency-meta">
                        <span class="latency-meta-label">"Latencia Red: "</span>
                        <span class="latency-badge">{move || latency_display.get()}</span>
                    </div>
                    <div class="data-age">
                        {move || age_text.get()}
                    </div>
                </div>
            </div>
        </section>
    }
}

#[component]
#[allow(clippy::too_many_lines)]
pub fn RoomMonitoring() -> impl IntoView {
    let (twin_state, set_twin_state) = create_signal(HashMap::<String, DeviceState>::new());
    let (current_time, set_current_time) = create_signal(Date::now());
    let (jwt_token, set_jwt_token) = create_signal(None::<String>);
    let (dlq_records, set_dlq_records) = create_signal(Vec::<DlqRecordDto>::new());
    let (dlq_state, set_dlq_state) = create_signal(DlqFetchState::Idle);
    let (parse_errors, set_parse_errors) = create_signal(0usize);

    // Consumir el estado de conexión global compartido
    let conn_status = use_context::<RwSignal<ConnectionStatus>>()
        .unwrap_or_else(|| create_rw_signal(ConnectionStatus::Disconnected));

    // Desacoplar IDs de dispositivos en un Memo ordenado determinísticamente
    let device_ids = create_memo(move |_| {
        twin_state.with(|map| {
            let mut ids: Vec<String> = map.keys().cloned().collect();
            ids.sort();
            ids
        })
    });

    // Timer para actualización de tiempo de staleness (1 segundo)
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

    let (operator_key, set_operator_key) = create_signal(String::new());
    let (login_error, set_login_error) = create_signal(None::<String>);

    let login_operator = create_action(move |key: &String| {
        let key_str = key.clone();
        async move {
            set_login_error.set(None);
            let login_endpoint = api_url("/api/auth/login");
            let client = reqwest::Client::new();
            match client.post(&login_endpoint).header("x-api-key", &key_str).send().await {
                Ok(res) => {
                    let status = res.status();
                    if status.is_success() {
                        if let Ok(json) = res.json::<serde_json::Value>().await {
                            if let Some(token) = json.get("token").and_then(|t| t.as_str()) {
                                let token_str = token.to_string();
                                set_jwt_token.set(Some(token_str.clone()));
                                set_operator_key.set(String::new());
                                set_dlq_state.set(DlqFetchState::Loading);

                                // Consulta inmediata al desbloquear consola
                                let dlq_endpoint = api_url("/api/dlq/recent");
                                if let Ok(dlq_res) = client
                                    .get(&dlq_endpoint)
                                    .header("Authorization", format!("Bearer {token_str}"))
                                    .send()
                                    .await
                                {
                                    if dlq_res.status().is_success() {
                                        if let Ok(records) =
                                            dlq_res.json::<Vec<DlqRecordDto>>().await
                                        {
                                            set_dlq_records.set(records);
                                            set_dlq_state.set(DlqFetchState::Ok);
                                        }
                                    }
                                }
                            }
                        }
                    } else if status == reqwest::StatusCode::UNAUTHORIZED {
                        set_login_error.set(Some(
                            "Clave de operador no válida (401 Unauthorized)".to_string(),
                        ));
                    } else {
                        set_login_error.set(Some(format!("Error de autenticación: {status}")));
                    }
                }
                Err(e) => {
                    set_login_error.set(Some(format!("Error de red: {e}")));
                }
            }
        }
    });

    // Bandera de cancelación compartida para detener workers asíncronos en on_cleanup
    let is_cancelled = Rc::new(Cell::new(false));
    let is_cancelled_cleanup = is_cancelled.clone();

    on_cleanup(move || {
        is_cancelled_cleanup.set(true);
        conn_status.set(ConnectionStatus::Disconnected);
    });

    // DLQ Polling (cada 5 segundos, sólo cuando el operador está autenticado)
    let is_cancelled_dlq = is_cancelled.clone();
    create_effect(move |_| {
        let is_cancelled_interval = is_cancelled_dlq.clone();
        let handle = leptos::set_interval_with_handle(
            move || {
                let Some(token) = jwt_token.get() else {
                    return;
                };

                let is_cancelled = is_cancelled_interval.clone();
                spawn_local(async move {
                    if is_cancelled.get() {
                        return;
                    }
                    set_dlq_state.set(DlqFetchState::Loading);
                    let dlq_endpoint = api_url("/api/dlq/recent");
                    let client = reqwest::Client::new();
                    match client
                        .get(&dlq_endpoint)
                        .header("Authorization", format!("Bearer {token}"))
                        .send()
                        .await
                    {
                        Ok(res) => {
                            let status = res.status();
                            if status.is_success() {
                                match res.json::<Vec<DlqRecordDto>>().await {
                                    Ok(records) => {
                                        set_dlq_records.set(records);
                                        set_dlq_state.set(DlqFetchState::Ok);
                                    }
                                    Err(e) => {
                                        set_dlq_state.set(DlqFetchState::Err(format!(
                                            "Error deserializando registros DLQ: {e}"
                                        )));
                                    }
                                }
                            } else if status == reqwest::StatusCode::UNAUTHORIZED
                                || status == reqwest::StatusCode::FORBIDDEN
                            {
                                set_jwt_token.set(None);
                                set_dlq_state.set(DlqFetchState::Err(
                                    "Sesión de operador expirada (HTTP 401/403). Inicie sesión nuevamente.".to_string(),
                                ));
                            } else {
                                set_dlq_state.set(DlqFetchState::Err(format!(
                                    "Fallo en API DLQ: HTTP {status}"
                                )));
                            }
                        }
                        Err(e) => {
                            set_dlq_state.set(DlqFetchState::Err(format!(
                                "Error de red al consultar DLQ: {e}"
                            )));
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

    // Worker WebSocket resiliente con backoff exponencial y cancelación limpia
    let is_cancelled_ws = is_cancelled;
    spawn_local(async move {
        let mut backoff_secs = 1u64;

        while !is_cancelled_ws.get() {
            conn_status.set(ConnectionStatus::Connecting);
            let target_ws = ws_url("/api/ws");

            match WebSocket::open(&target_ws) {
                Ok(ws) => {
                    conn_status.set(ConnectionStatus::Connected);
                    backoff_secs = 1; // Restablecer backoff tras conexión exitosa
                    let (_, mut read) = ws.split();

                    while let Some(msg) = read.next().await {
                        if is_cancelled_ws.get() {
                            break;
                        }
                        match msg {
                            Ok(Message::Text(text)) => {
                                match serde_json::from_str::<TelemetryPayload>(&text) {
                                    Ok(payload) => {
                                        set_twin_state.update(|state| {
                                            let (mut temp_history, mut co2_history) =
                                                state.get(&payload.device_id).map_or_else(
                                                    || {
                                                        (
                                                            VecDeque::with_capacity(60),
                                                            VecDeque::with_capacity(60),
                                                        )
                                                    },
                                                    |existing| {
                                                        (
                                                            existing.temp_history.clone(),
                                                            existing.co2_history.clone(),
                                                        )
                                                    },
                                                );

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
                                    Err(err) => {
                                        set_parse_errors.update(|count| *count += 1);
                                        leptos::logging::warn!(
                                            "Error deserializando telemetría JSON: {err}"
                                        );
                                    }
                                }
                            }
                            Ok(Message::Bytes(_)) => {}
                            Err(e) => {
                                leptos::logging::warn!("Error en stream WebSocket: {e:?}");
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    leptos::logging::warn!("No se pudo abrir canal WebSocket: {e:?}");
                }
            }

            if is_cancelled_ws.get() {
                break;
            }

            conn_status.set(ConnectionStatus::Disconnected);

            // Backoff exponencial: 1s, 2s, 4s, 8s, 16s, máx 30s
            let delay_ms = backoff_secs * 1000;
            backoff_secs = (backoff_secs * 2).min(30);

            let (tx, rx) = futures::channel::oneshot::channel::<()>();
            let handle = leptos::set_timeout_with_handle(
                move || {
                    let _ = tx.send(());
                },
                std::time::Duration::from_millis(delay_ms),
            );

            let _ = rx.await;
            if let Ok(h) = handle {
                h.clear();
            }
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
                            {if parse_errors.get() > 0 {
                                view! {
                                    <span class="status-parse-errors tabular-nums" role="alert" title="Paquetes corruptos detectados">
                                        {format!("⚠ Parse Errors: {}", parse_errors.get())}
                                    </span>
                                }.into_any()
                            } else {
                                view! { <span/> }.into_any()
                            }}
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
                when=move || twin_state.with(|m| !m.is_empty())
                fallback=move || view! {
                    <div class="status-waiting">
                        <div class="spinner"></div>
                        <h3 class="pulse-text">"Esperando telemetría de dispositivos de campo..."</h3>
                        <p>"(Subscriptor GCP Pub/Sub y Gemelo Digital en Memoria)"</p>
                    </div>
                }
            >
                <div class="telemetry-fleet-grid">
                    <For
                        each=move || device_ids.get()
                        key=|id| id.clone()
                        children=move |id| {
                            view! {
                                <NodeCard
                                    device_id=id
                                    twin_state=twin_state
                                    current_time=current_time
                                />
                            }
                        }
                    />
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
                            when=move || jwt_token.get().is_some()
                            fallback=move || view! {
                                <div class="dlq-locked-box">
                                    <p class="dlq-locked-msg">"[LOCKED / ZERO-TRUST] Ingrese clave de operador para inspeccionar payloads crudos de la DLQ"</p>
                                    <div class="dlq-auth-form">
                                        <label for="dlq-operator-key-input" class="dlq-key-label">
                                            "Clave de Operador (ADMIN_API_KEY):"
                                        </label>
                                        <input
                                            id="dlq-operator-key-input"
                                            type="password"
                                            class="dlq-key-input"
                                            placeholder="ADMIN_API_KEY..."
                                            prop:value=operator_key
                                            on:input=move |ev| set_operator_key.set(event_target_value(&ev))
                                            on:keydown=move |ev| {
                                                if ev.key() == "Enter" && !operator_key.get().trim().is_empty() {
                                                    login_operator.dispatch(operator_key.get());
                                                }
                                            }
                                        />
                                        <button
                                            type="button"
                                            class="dlq-unlock-btn"
                                            on:click=move |_| {
                                                if !operator_key.get().trim().is_empty() {
                                                    login_operator.dispatch(operator_key.get());
                                                }
                                            }
                                        >
                                            "DESBLOQUEAR"
                                        </button>
                                    </div>
                                    {move || login_error.get().map(|err| view! {
                                        <p class="dlq-login-error">{err}</p>
                                    })}
                                </div>
                            }
                        >
                            <div class="dlq-authenticated-header">
                                <div class="dlq-auth-status">
                                    <span class="dlq-auth-dot" aria-hidden="true"></span>
                                    <span class="dlq-auth-text">"OPERADOR AUTENTICADO (Zero-Trust Session)"</span>
                                </div>
                                <button
                                    type="button"
                                    class="dlq-lock-btn"
                                    on:click=move |_| {
                                        set_jwt_token.set(None);
                                        set_dlq_records.set(Vec::new());
                                        set_dlq_state.set(DlqFetchState::Idle);
                                    }
                                >
                                    "LOCK CONSOLE"
                                </button>
                            </div>

                            {move || match dlq_state.get() {
                                DlqFetchState::Err(err_msg) => view! {
                                    <div class="dlq-error-box" role="alert">
                                        <svg class="alert-svg-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                                            <circle cx="12" cy="12" r="10"></circle>
                                            <line x1="12" y1="8" x2="12" y2="12"></line>
                                            <line x1="12" y1="16" x2="12.01" y2="16"></line>
                                        </svg>
                                        <span class="dlq-error-text">{format!("[ERROR DLQ] {err_msg}")}</span>
                                    </div>
                                }.into_any(),
                                DlqFetchState::Loading if dlq_records.get().is_empty() => view! {
                                    <div class="dlq-loading-box">
                                        <span class="pulse-text">"Consultando registros DLQ..."</span>
                                    </div>
                                }.into_any(),
                                _ => {
                                    if dlq_records.get().is_empty() {
                                        view! {
                                            <p class="dlq-ok-msg">"[OK] No hay Poison Pills recientes en la cola."</p>
                                        }.into_any()
                                    } else {
                                        view! {
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
                                        }.into_any()
                                    }
                                }
                            }}
                        </Show>
                    </div>
                </div>
            </div>
        </div>
    }
}
