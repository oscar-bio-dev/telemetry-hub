// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.

use crate::components::spec_sheet::{SpecRow, SpecSheet};
use leptos::*;

/// Diagrama SVG del pipeline científico cerrado: In-situ -> Telemetría -> Modelado HPC
#[component]
fn DiagramFieldPipeline() -> impl IntoView {
    view! {
        <div class="terminal-card diagram-card">
            <div class="card-header diagram-header">
                <span class="dot red" aria-hidden="true"></span>
                <span class="dot yellow" aria-hidden="true"></span>
                <span class="dot green" aria-hidden="true"></span>
                <span class="filename">"scientific_pipeline::empirical_to_hpc.svg"</span>
            </div>
            <div class="card-body diagram-body">
                <svg class="block-diagram-svg" viewBox="0 0 760 130" fill="none" xmlns="http://www.w3.org/2000/svg" aria-label="Pipeline de Telemetría a Modelado">
                    // Bloque 1: Instrumentación In-Situ
                    <rect x="10" y="25" width="135" height="75" rx="4" class="diag-block block-accent-yellow" />
                    <text x="77" y="52" class="diag-text-title" text-anchor="middle">"Sensores In-Situ"</text>
                    <text x="77" y="68" class="diag-text-sub" text-anchor="middle">"Horizontes O, A, B"</text>
                    <text x="77" y="82" class="diag-text-detail" text-anchor="middle">"CO2, VWC, Temp, pH"</text>

                    // Flecha 1 -> 2
                    <path d="M145 62 L175 62" class="diag-arrow" />
                    <polygon points="175,58 183,62 175,66" class="diag-arrowhead" />

                    // Bloque 2: Nodos EcoWest
                    <rect x="183" y="25" width="130" height="75" rx="4" class="diag-block block-accent-green" />
                    <text x="248" y="52" class="diag-text-title" text-anchor="middle">"Nodos EcoWest"</text>
                    <text x="248" y="68" class="diag-text-sub" text-anchor="middle">"Bare-Metal C / Rust"</text>
                    <text x="248" y="82" class="diag-text-detail" text-anchor="middle">"Nanopb Protobuf"</text>

                    // Flecha 2 -> 3
                    <path d="M313 62 L343 62" class="diag-arrow" />
                    <polygon points="343,58 351,62 343,66" class="diag-arrowhead" />

                    // Bloque 3: Ingesta Cloud
                    <rect x="351" y="25" width="130" height="75" rx="4" class="diag-block block-accent-blue" />
                    <text x="416" y="52" class="diag-text-title" text-anchor="middle">"Telemetry Hub"</text>
                    <text x="416" y="68" class="diag-text-sub" text-anchor="middle">"GCP Pub/Sub + DLQ"</text>
                    <text x="416" y="82" class="diag-text-detail" text-anchor="middle">"TimescaleDB Hypertable"</text>

                    // Flecha 3 -> 4
                    <path d="M481 62 L511 62" class="diag-arrow" />
                    <polygon points="511,58 519,62 511,66" class="diag-arrowhead" />

                    // Bloque 4: Asimilación AMYNTHAS-HPC
                    <rect x="519" y="15" width="230" height="95" rx="4" class="diag-block block-accent-main" />
                    <text x="634" y="42" class="diag-text-title" text-anchor="middle">"AMYNTHAS-HPC Engine"</text>
                    <text x="634" y="58" class="diag-text-sub" text-anchor="middle">"Calibración de Parámetros D y r"</text>
                    <text x="634" y="74" class="diag-text-detail" text-anchor="middle">"Asimilación Continua de Datos"</text>
                    <text x="634" y="88" class="diag-text-detail" text-anchor="middle">"Predicción de Mineralización y Carbono"</text>
                </svg>
            </div>
        </div>
    }
}

#[component]
pub fn FieldResearchPage() -> impl IntoView {
    let research_specs = vec![
        SpecRow::new(
            "Localización de Campo",
            "Pueblo West, Colorado, EE.UU.",
            "Coordenadas Semiáridas",
            "Elevación 1,510 m",
            "Gradiente de temperatura -25 °C a +42 °C",
        ),
        SpecRow::new(
            "Perfiles de Profundidad",
            "Horizontes O (0-10cm), A (10-30cm), B (30-60cm)",
            "3 niveles",
            "Sondas multinivel",
            "Contenido volumétrico de agua y flujo de gas",
        ),
        SpecRow::new(
            "Frecuencia de Muestreo",
            "60 segundos por canal",
            "Tiempo continuo",
            "Precisión < 1 ms",
            "Agregación y compresión en TimescaleDB",
        ),
        SpecRow::new(
            "Especies de Estudio",
            "Amynthas agrestis, Lumbricus terrestris",
            "Taxonomía biológica",
            "Invasora vs. Autóctona",
            "Tasas de bioturbación y consumo de mantillo",
        ),
        SpecRow::new(
            "Parámetros Físico-Químicos",
            "VWC, pH, Potencial Redox, CO2 edáfico",
            "Multi-parámetro",
            "Sensores estado sólido",
            "Compensación de presión barométrica en vivo",
        ),
        SpecRow::new(
            "Enlace con Simulación",
            "Asimilación Bayesiana a AMYNTHAS-HPC",
            "Pipeline continuo",
            "Validación cruzada",
            "Ajuste dinámico de difusión D(T, VWC) y crecimiento r",
        ),
    ];

    view! {
        <div class="page-container field-research-page">
            <header class="section-header">
                <h2 class="section-title">"~/field-research - Biología Empírica e Investigación Edáfica"</h2>
                <p class="section-desc">
                    "Intersección entre la biología de campo, la taxonomía de oligoquetos y la instrumentación electrónica para alimentar modelos ecológicos predictivos en Colorado."
                </p>
            </header>

            // Sección 1: Diagrama de Pipeline de Datos Empíricos a HPC
            <section class="research-pipeline-section">
                <h3 class="section-title">"Pipeline Cerrado: De la Biología al Modelado HPC"</h3>
                <DiagramFieldPipeline />
            </section>

            // Sección 2: Tarjetas de Dominios Científicos
            <section class="research-cards-grid">
                <article class="terminal-card research-card">
                    <div class="card-header">
                        <span class="dot red" aria-hidden="true"></span>
                        <span class="dot yellow" aria-hidden="true"></span>
                        <span class="dot green" aria-hidden="true"></span>
                        <span class="filename">"field_study::soil_biodiversity_dynamics.md"</span>
                    </div>
                    <div class="card-body">
                        <h4 class="research-card-title">"1. Biodiversidad Edáfica y Respiración del Suelo"</h4>
                        <p>
                            "El monitoreo continuo de la concentración de CO2 en la matriz de poros del suelo y su correlación con la humedad volumétrica (VWC) permite aislar la respiración microbiana basal de la respiración inducida por macroinvertebrados."
                        </p>
                        <ul class="research-bullet-list">
                            <li><strong>"Gradiente Térmico:"</strong>" Medición simultánea en capas superficiales (0-10 cm) y horizontes radiculares profundos (30-60 cm)."</li>
                            <li><strong>"Mineralización de Carbono:"</strong>" Estimación del impacto de la perturbación edáfica en la pérdida de carbono orgánico hacia la atmósfera."</li>
                        </ul>
                    </div>
                </article>

                <article class="terminal-card research-card">
                    <div class="card-header">
                        <span class="dot red" aria-hidden="true"></span>
                        <span class="dot yellow" aria-hidden="true"></span>
                        <span class="dot green" aria-hidden="true"></span>
                        <span class="filename">"field_study::oligochaeta_taxonomy.md"</span>
                    </div>
                    <div class="card-body">
                        <h4 class="research-card-title">"2. Taxonomía de Oligoquetos y Especies Invasoras"</h4>
                        <p>
                            "Estudio taxonómico de la dispersión de "<em class="taxon-name">"Amynthas agrestis"</em>" (Crazy Snake Worm / Asian Jumping Worm), una especie anécica invasora que desestabiliza la estructura física del horizonte orgánico del suelo en América del Norte."
                        </p>
                        <ul class="research-bullet-list">
                            <li><strong>"Morfometría y Clímax:"</strong>" Clasificación del clitelo, setas periquetinas y poros dorsales para determinar estadios de desarrollo en campo."</li>
                            <li><strong>"Vectores de Dispersión:"</strong>" Cuantificación de capullos viables en suelos áridos y modelado de frentes de onda biológicos."</li>
                        </ul>
                    </div>
                </article>

                <article class="terminal-card research-card">
                    <div class="card-header">
                        <span class="dot red" aria-hidden="true"></span>
                        <span class="dot yellow" aria-hidden="true"></span>
                        <span class="dot green" aria-hidden="true"></span>
                        <span class="filename">"field_study::rugged_instrumentation.md"</span>
                    </div>
                    <div class="card-body">
                        <h4 class="research-card-title">"3. Resiliencia de Instrumentación en Condiciones Extremas"</h4>
                        <p>
                            "Validación empírica del hardware EcoWest Instrument en el clima continental semiárido de Pueblo West, Colorado, caracterizado por oscilaciones térmicas extremas (-25 °C en invierno a +42 °C en verano)."
                        </p>
                        <ul class="research-bullet-list">
                            <li><strong>"Encapsulado NEMA 4X / IP67:"</strong>" Alivio de presión mediante membranas hidrófobas Gore-Tex para prevenir condensación interna."</li>
                            <li><strong>"Cosecha Energética Solar:"</strong>" Baterías LiFePO4 de alta estabilidad química con perfiles de carga compensados por temperatura."</li>
                        </ul>
                    </div>
                </article>
            </section>

            // Sección 3: Especificaciones del Protocolo de Campo
            <section class="research-specs-section">
                <SpecSheet
                    title="Protocolo Metrológico de Investigación Edáfica"
                    subtitle="Estándares de calibración, profundidades de sondeo y tolerancias de instrumentación in-situ."
                    specs=research_specs
                />
            </section>
        </div>
    }
}
