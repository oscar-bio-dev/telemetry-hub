// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.

use crate::components::spec_sheet::{SpecRow, SpecSheet};
use leptos::*;
use leptos_router::*;

/// Icono técnico SVG: Microcontrolador / Chip de Procesamiento
#[component]
fn IconMicrocontroller() -> impl IntoView {
    view! {
        <svg class="tech-icon" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <rect x="4" y="4" width="16" height="16" rx="2" />
            <rect x="9" y="9" width="6" height="6" />
            <line x1="9" y1="1" x2="9" y2="4" />
            <line x1="12" y1="1" x2="12" y2="4" />
            <line x1="15" y1="1" x2="15" y2="4" />
            <line x1="9" y1="20" x2="9" y2="23" />
            <line x1="12" y1="20" x2="12" y2="23" />
            <line x1="15" y1="20" x2="15" y2="23" />
            <line x1="1" y1="9" x2="4" y2="9" />
            <line x1="1" y1="12" x2="4" y2="12" />
            <line x1="1" y1="15" x2="4" y2="15" />
            <line x1="20" y1="9" x2="23" y2="9" />
            <line x1="20" y1="12" x2="23" y2="12" />
            <line x1="20" y1="15" x2="23" y2="15" />
        </svg>
    }
}

/// Icono técnico SVG: Sonda Óptica de Medición
#[component]
fn IconOpticalProbe() -> impl IntoView {
    view! {
        <svg class="tech-icon" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <line x1="12" y1="1" x2="12" y2="4" />
            <rect x="7" y="4" width="10" height="11" rx="2" />
            <path d="M7 15 L12 20 L17 15" />
            <line x1="12" y1="20" x2="12" y2="23" />
            <line x1="9" y1="19" x2="7" y2="22" />
            <line x1="15" y1="19" x2="17" y2="22" />
            <circle cx="12" cy="9.5" r="1.5" />
        </svg>
    }
}

/// Icono técnico SVG: Nodo RS485 / Bus Industrial
#[component]
fn IconRs485Node() -> impl IntoView {
    view! {
        <svg class="tech-icon" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <rect x="3" y="6" width="18" height="12" rx="2" />
            <line x1="1" y1="9" x2="3" y2="9" />
            <line x1="1" y1="15" x2="3" y2="15" />
            <line x1="21" y1="9" x2="23" y2="9" />
            <line x1="21" y1="15" x2="23" y2="15" />
            <path d="M7 12 h3 l2 -3 l2 6 l2 -3 h2" />
        </svg>
    }
}

/// Icono técnico SVG: Malla de Simulación / Vector
#[component]
fn IconMeshSimulation() -> impl IntoView {
    view! {
        <svg class="tech-icon" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M2 12 L12 2 L22 12 L12 22 Z" />
            <line x1="12" y1="2" x2="12" y2="22" />
            <line x1="2" y1="12" x2="22" y2="12" />
            <circle cx="12" cy="12" r="2.5" />
        </svg>
    }
}

#[component]
pub fn Home() -> impl IntoView {
    let telemetry_specs = vec![
        SpecRow::new(
            "Protocolo Wire",
            "Nanopb / Protocol Buffers v2.0",
            "Mega-Schema",
            "33 campos canónicos",
            "Compacto f32 en radio, f64 en BD",
        ),
        SpecRow::new(
            "Ingestión Asíncrona",
            "GCP Pub/Sub Pull Worker",
            "Tokio Async",
            "Tolerancia ante cortes",
            "Enrutamiento DLQ para Poison Pills",
        ),
        SpecRow::new(
            "Gemelo Digital",
            "Arc<RwLock<LruCache>>",
            "Submilisegundo",
            "Tope 10,000 nodos",
            "Sincronización en vivo con HMI",
        ),
        SpecRow::new(
            "Persistencia Temporal",
            "TimescaleDB Hypertables",
            "SQLx PostgreSQL",
            "Chunks continuos",
            "Compresión de series temporales",
        ),
    ];

    let hpc_summary_specs = vec![
        SpecRow::new(
            "Núcleo Matemático",
            "Modern Fortran 2018 + C++20",
            "Estándar ISO",
            "0 runtime cost",
            "Ecuaciones ADRE espacio-temporales",
        ),
        SpecRow::new(
            "Vectorización SIMD",
            "Intel AVX-512 F/CD/BW/DQ/VL",
            "512 bits",
            "8 doubles/ciclo",
            "Optimizaciones de microarquitectura",
        ),
        SpecRow::new(
            "Datos Geoespaciales",
            "PostgreSQL 16 + PostGIS",
            "GIST Spatial",
            "Indexación ráster",
            "Acoplamiento con microcuencas",
        ),
    ];

    let hw_summary_specs = vec![
        SpecRow::new(
            "Sonda Óptica DO",
            "EcoWest WQM-V100",
            "Luminiscencia",
            "0.0 - 20.0 mg/L",
            "Decaimiento de fase, sin consumo O2",
        ),
        SpecRow::new(
            "Gateway Criptográfico",
            "ESP32-P4 Dual-Core RISC-V",
            "400 MHz",
            "ECDSA ES256",
            "Ingesta ESP-NOW y enlace celular",
        ),
        SpecRow::new(
            "Consumo Energético",
            "Nodos Sensores Bare-Metal",
            "Microamperios",
            "1.5 µA deep sleep",
            "Alimentación solar LiFePO4 de campo",
        ),
    ];

    view! {
        <div class="page-container executive-dashboard">
            // Header Ejecutivo con Perfil Técnico
            <section class="terminal-card profile-executive-card">
                <div class="card-header">
                    <span class="dot red" aria-hidden="true"></span>
                    <span class="dot yellow" aria-hidden="true"></span>
                    <span class="dot green" aria-hidden="true"></span>
                    <span class="filename">"executive_overview::oscar_mora.md"</span>
                </div>
                <div class="card-body">
                    <div class="executive-summary-grid">
                        <div class="executive-bio">
                            <h2 class="executive-title">"Oscar Mora"</h2>
                            <p class="executive-role">"Conservation Technologist, Marine Biologist & Embedded Systems Engineer"</p>
                            <p class="executive-org">"Fundador de "<strong class="brand-text">"EcoWest Instrument"</strong>" | Pueblo West, Colorado"</p>
                            <p class="executive-statement">
                                "Diseño y fabricación de instrumentación ambiental de grado industrial, arquitectura de sensores bare-metal y plataformas de telemetría resilientes edge-to-cloud para ecología de campo y modelado computacional."
                            </p>
                        </div>
                        <div class="executive-links-panel">
                            <h3 class="panel-subtitle">"Redes Profesionales & Código"</h3>
                            <div class="links-button-stack">
                                <a href="https://github.com/oscar-bio-dev" target="_blank" rel="noopener noreferrer" class="exec-link-btn">
                                    <span class="btn-prompt">"gh"</span>
                                    <span>"github.com/oscar-bio-dev"</span>
                                </a>
                                <a href="https://github.com/oscar-bio-dev/telemetry-hub" target="_blank" rel="noopener noreferrer" class="exec-link-btn exec-link-highlight">
                                    <span class="btn-prompt">"repo"</span>
                                    <span>"telemetry-hub (Rust Core)"</span>
                                </a>
                                <a href="https://oscar-bio.dev/" target="_blank" rel="noopener noreferrer" class="exec-link-btn">
                                    <span class="btn-prompt">"web"</span>
                                    <span>"EcoWest Instrument"</span>
                                </a>
                                <a href="https://linkedin.com/in/oscar-bio-dev" target="_blank" rel="noopener noreferrer" class="exec-link-btn">
                                    <span class="btn-prompt">"in"</span>
                                    <span>"linkedin.com/in/oscar-bio-dev"</span>
                                </a>
                            </div>
                        </div>
                    </div>
                </div>
            </section>

            // Módulos del Sistema SCADA HMI con Iconografía SVG Técnica
            <section class="systems-hub-section">
                <h2 class="section-title">"Módulos Técnicos del Ecosistema"</h2>
                <div class="project-grid">
                    <A href="/telemetry/room" class="project-item item-telemetry">
                        <span class="project-tech-icon icon-live">
                            <IconRs485Node />
                        </span>
                        <div class="project-info">
                            <div class="project-header-line">
                                <h3 class="project-heading-telemetry">"~/telemetry/room - HMI en Tiempo Real"</h3>
                                <span class="badge-live-stream">"● STREAMING"</span>
                            </div>
                            <p>"Monitoreo continuo vía WebSockets del Gemelo Digital en memoria, buffers de 60 muestras por nodo y sparklines SVG de tendencia."</p>
                        </div>
                    </A>

                    <A href="/hardware" class="project-item">
                        <span class="project-tech-icon">
                            <IconOpticalProbe />
                        </span>
                        <div class="project-info">
                            <h3>"~/hardware - Instrumentación EcoWest"</h3>
                            <p>"Medidor óptico DO WQM-V100 (fluorometría de fase), gateways ESP32-P4/C6 y nodos ambientales BME688/SCD41/BMV080."</p>
                        </div>
                    </A>

                    <A href="/hpc" class="project-item">
                        <span class="project-tech-icon">
                            <IconMeshSimulation />
                        </span>
                        <div class="project-info">
                            <h3>"~/hpc-modeling - AMYNTHAS-HPC"</h3>
                            <p>"Modelado computacional de dispersión de especies invasoras y flujos de carbono en Fortran 2018/C++ con vectorización AVX-512 y PostGIS."</p>
                        </div>
                    </A>

                    <A href="/field-research" class="project-item">
                        <span class="project-tech-icon">
                            <IconMicrocontroller />
                        </span>
                        <div class="project-info">
                            <h3>"~/field-research - Investigación Edáfica y Biológica"</h3>
                            <p>"Estudios taxonómicos de oligoquetos y campañas de monitoreo in-situ en Colorado correlacionando microclima con modelos biológicos."</p>
                        </div>
                    </A>
                </div>
            </section>

            // Tarjetas Técnicas con Resumen de Especificaciones
            <section class="domain-datasheets-section">
                <h2 class="section-title">"Especificaciones de Arquitectura y Rendimiento"</h2>
                <div class="specsheets-stack">
                    <SpecSheet
                        title="Telemetría Edge-to-Cloud (Mega-Schema v2.0)"
                        subtitle="Pipeline asíncrono en Rust Axum, GCP Pub/Sub y TimescaleDB."
                        specs=telemetry_specs
                    />

                    <SpecSheet
                        title="AMYNTHAS-HPC (Cálculo Vectorial)"
                        subtitle="Núcleo de alto rendimiento numérico en Fortran 2018 con AVX-512 y aceleración SIMD."
                        specs=hpc_summary_specs
                    />

                    <SpecSheet
                        title="Hardware e Instrumentación Bare-Metal"
                        subtitle="Transducción óptica, bajo consumo y electrónica de potencia LiFePO4."
                        specs=hw_summary_specs
                    />
                </div>
            </section>
        </div>
    }
}
