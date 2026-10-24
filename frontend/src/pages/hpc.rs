// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.

use crate::components::spec_sheet::{SpecRow, SpecSheet};
use leptos::*;

const GRID_SIZE: usize = 16;
const TOTAL_CELLS: usize = GRID_SIZE * GRID_SIZE;

fn init_hotspot_grid() -> Vec<f64> {
    let mut grid = vec![0.0; TOTAL_CELLS];
    // Inocular foco central inicial de especie invasora
    let center = GRID_SIZE / 2;
    grid[(center - 1) * GRID_SIZE + (center - 1)] = 0.85;
    grid[(center - 1) * GRID_SIZE + center] = 0.95;
    grid[center * GRID_SIZE + (center - 1)] = 0.95;
    grid[center * GRID_SIZE + center] = 0.85;
    grid
}

/// Widget interactivo de simulación numérica de Reacción-Difusión (ADRE) en Wasm puro
#[component]
fn HpcGridSimulation() -> impl IntoView {
    let (grid, set_grid) = create_signal(init_hotspot_grid());
    let (diffusion, set_diffusion) = create_signal(0.18_f64);
    let (growth_rate, set_growth_rate) = create_signal(0.06_f64);
    let (step_count, set_step_count) = create_signal(0_usize);
    let (is_running, set_is_running) = create_signal(false);

    // Función pura de avance temporal (Kernel de Diferencias Finitas)
    let advance_step = move || {
        let d = diffusion.get();
        let r = growth_rate.get();
        let dt = 0.5_f64;
        let k = 1.0_f64; // Capacidad de carga edáfica normalizada

        set_grid.update(|curr| {
            let mut next = curr.clone();
            for y in 0..GRID_SIZE {
                for x in 0..GRID_SIZE {
                    let idx = y * GRID_SIZE + x;
                    let u = curr[idx];

                    // Condiciones de contorno reflectivas tipo Neumann (d_u/d_n = 0)
                    let left = if x > 0 { curr[y * GRID_SIZE + (x - 1)] } else { u };
                    let right = if x + 1 < GRID_SIZE { curr[y * GRID_SIZE + (x + 1)] } else { u };
                    let up = if y > 0 { curr[(y - 1) * GRID_SIZE + x] } else { u };
                    let down = if y + 1 < GRID_SIZE { curr[(y + 1) * GRID_SIZE + x] } else { u };

                    let laplacian = left + right + up + down - (4.0 * u);
                    let diff_term = d * laplacian;
                    let react_term = r * u * (1.0 - (u / k));

                    let updated = u + dt * (diff_term + react_term);
                    next[idx] = updated.clamp(0.0, 1.25);
                }
            }
            *curr = next;
        });
        set_step_count.update(|c| *c += 1);
    };

    // Temporizador de ejecución continua reactiva en Leptos
    create_effect(move |_| {
        if is_running.get() {
            let handle = leptos::set_interval_with_handle(
                move || {
                    advance_step();
                },
                std::time::Duration::from_millis(160),
            );
            on_cleanup(move || {
                if let Ok(h) = handle {
                    h.clear();
                }
            });
        }
    });

    let reset_grid = move |_| {
        set_grid.set(init_hotspot_grid());
        set_step_count.set(0);
        set_is_running.set(false);
    };

    let inoculate_extra = move |_| {
        set_grid.update(|curr| {
            // Inocular foco periférico secundario
            let p1 = 3 * GRID_SIZE + 4;
            let p2 = 12 * GRID_SIZE + 11;
            curr[p1] = 0.90;
            curr[p2] = 0.85;
        });
    };

    // Cálculo reactivo de métricas agregadas de la malla
    let metrics = move || {
        let cells = grid.get();
        let total: f64 = cells.iter().sum();
        let max_val: f64 = cells.iter().copied().fold(0.0, f64::max);
        let occupied = cells.iter().filter(|&&v| v > 0.05).count();
        let coverage_pct = (occupied as f64 / TOTAL_CELLS as f64) * 100.0;
        (total, max_val, coverage_pct)
    };

    view! {
        <div class="terminal-card sim-widget-card">
            <div class="card-header sim-header">
                <span class="dot red" aria-hidden="true"></span>
                <span class="dot yellow" aria-hidden="true"></span>
                <span class="dot green" aria-hidden="true"></span>
                <span class="filename">"sim_kernel::adre_2d_mesh.wasm"</span>
                <span class="sim-badge tabular-nums">{move || format!("PASO: #{}", step_count.get())}</span>
            </div>
            <div class="card-body sim-body">
                <div class="sim-layout-grid">
                    // Malla 2D Vectorial en SVG
                    <div class="sim-canvas-panel">
                        <svg class="sim-svg-grid" viewBox="0 0 320 320" xmlns="http://www.w3.org/2000/svg" aria-label="Malla de Simulación Reacción-Difusión">
                            {move || {
                                let cells = grid.get();
                                (0..TOTAL_CELLS).map(|idx| {
                                    let x = (idx % GRID_SIZE) * 20;
                                    let y = (idx / GRID_SIZE) * 20;
                                    let val = cells[idx];

                                    // Rampa de color Gruvbox según densidad de biomasa u
                                    let fill_color = if val < 0.02 {
                                        "#282828"
                                    } else if val < 0.25 {
                                        "#458588" // Azul grisáceo
                                    } else if val < 0.55 {
                                        "#b8bb26" // Verde
                                    } else if val < 0.85 {
                                        "#fabd2f" // Amarillo
                                    } else {
                                        "#fb4934" // Rojo advertencia
                                    };

                                    let opacity = if val < 0.02 {
                                        "0.4"
                                    } else {
                                        "0.9"
                                    };

                                    view! {
                                        <rect
                                            x=x
                                            y=y
                                            width="19"
                                            height="19"
                                            rx="2"
                                            fill=fill_color
                                            opacity=opacity
                                            stroke="#1d2021"
                                            stroke-width="1"
                                        />
                                    }
                                }).collect_view()
                            }}
                        </svg>
                        <div class="sim-legend">
                            <span class="legend-item"><span class="legend-box col-zero"></span>"0.0 (Suelo Estéril)"</span>
                            <span class="legend-item"><span class="legend-box col-low"></span>"0.25 (Dispersión)"</span>
                            <span class="legend-item"><span class="legend-box col-mid"></span>"0.55 (Establecido)"</span>
                            <span class="legend-item"><span class="legend-box col-high"></span>"1.0+ (Saturación K)"</span>
                        </div>
                    </div>

                    // Panel de Controles y Sliders
                    <div class="sim-controls-panel">
                        <h4 class="controls-title">"Parámetros Dinámicos del Modelo"</h4>

                        <div class="control-row">
                            <div class="control-label-row">
                                <label for="slider-diffusion">"Coeficiente de Difusión (D)"</label>
                                <span class="control-val tabular-nums">{move || format!("{:.3}", diffusion.get())}</span>
                            </div>
                            <input
                                id="slider-diffusion"
                                type="range"
                                min="0.02"
                                max="0.40"
                                step="0.01"
                                prop:value=move || diffusion.get()
                                on:input=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                                        set_diffusion.set(val);
                                    }
                                }
                                class="sim-slider"
                            />
                        </div>

                        <div class="control-row">
                            <div class="control-label-row">
                                <label for="slider-growth">"Tasa Intrínseca de Crecimiento (r)"</label>
                                <span class="control-val tabular-nums">{move || format!("{:.3}", growth_rate.get())}</span>
                            </div>
                            <input
                                id="slider-growth"
                                type="range"
                                min="0.01"
                                max="0.20"
                                step="0.005"
                                prop:value=move || growth_rate.get()
                                on:input=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                                        set_growth_rate.set(val);
                                    }
                                }
                                class="sim-slider"
                            />
                        </div>

                        // Lectura de Métricas en Vivo
                        <div class="sim-stats-readout">
                            <div class="stat-box">
                                <span class="stat-lbl">"BIOMASA TOTAL"</span>
                                <span class="stat-num tabular-nums">{move || format!("{:.1}", metrics().0)}</span>
                            </div>
                            <div class="stat-box">
                                <span class="stat-lbl">"DENSIDAD MÁXIMA"</span>
                                <span class="stat-num tabular-nums">{move || format!("{:.2}", metrics().1)}</span>
                            </div>
                            <div class="stat-box">
                                <span class="stat-lbl">"COBERTURA"</span>
                                <span class="stat-num tabular-nums">{move || format!("{:.1}%", metrics().2)}</span>
                            </div>
                        </div>

                        // Botonera de Acción
                        <div class="sim-actions-grid">
                            <button
                                type="button"
                                on:click=move |_| advance_step()
                                class="sim-btn sim-btn-step"
                            >
                                "Paso (+1)"
                            </button>

                            <button
                                type="button"
                                on:click=move |_| set_is_running.update(|r| *r = !*r)
                                class=move || if is_running.get() { "sim-btn sim-btn-pause" } else { "sim-btn sim-btn-run" }
                            >
                                {move || if is_running.get() { "⏸ Pausar" } else { "▶ Ejecutar" }}
                            </button>

                            <button
                                type="button"
                                on:click=inoculate_extra
                                class="sim-btn sim-btn-inoculate"
                            >
                                "+ Inocular Focos"
                            </button>

                            <button
                                type="button"
                                on:click=reset_grid
                                class="sim-btn sim-btn-reset"
                            >
                                "↺ Reiniciar"
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn HpcPage() -> impl IntoView {
    let hpc_specs = vec![
        SpecRow::new(
            "Lenguaje del Núcleo",
            "Modern Fortran (2018) + C++20",
            "Estándar ISO",
            "Cero overhead de runtime",
            "Interoperabilidad nativa ISO_C_BINDING",
        ),
        SpecRow::new(
            "Aceleración Vectorial",
            "Intel AVX-512 F/CD/BW/DQ/VL",
            "SIMD 512-bit",
            "8 doubles / 16 floats por ciclo",
            "Optimizaciones de microarquitectura y loop unrolling",
        ),
        SpecRow::new(
            "Paralelismo de Memoria",
            "OpenMP 5.2 Offload + oneTBB",
            "Multi-core / NUMA",
            "Escalabilidad multinúcleo lineal",
            "Afinidad de hilos y balanceo dinámico de carga",
        ),
        SpecRow::new(
            "Modelado de Dispersión",
            "ADRE (Advection-Diffusion-Reaction)",
            "PDEs de 2do orden",
            "Esquemas conservativos Crank-Nicolson",
            "Simulación espacio-temporal continua edáfica",
        ),
        SpecRow::new(
            "Indexación Espacial",
            "PostgreSQL 16 + PostGIS",
            "Geom / GIST",
            "Consultas espaciales indexadas",
            "Mapeo ráster y vectorial de microcuencas",
        ),
        SpecRow::new(
            "Integración con Telemetría",
            "TimescaleDB Time-Series Feed",
            "Pipeline continuo",
            "Ingestión en microsegundos",
            "Asimilación de datos en tiempo real para calibración",
        ),
    ];

    view! {
        <div class="page-container hpc-page">
            <header class="section-header">
                <h2 class="section-title">"~/hpc-modeling - Proyecto AMYNTHAS-HPC"</h2>
                <p class="section-desc">
                    "Simulación computacional de alta fidelidad para dispersión de especies invasoras (Amynthas agrestis) y partición de carbono edáfico en Fortran moderno y C++ con vectorización AVX-512 y PostGIS."
                </p>
            </header>

            // Sección 1: Ecuación Matemática en MathML Nativo
            <section class="terminal-card mathml-card">
                <div class="card-header">
                    <span class="dot red" aria-hidden="true"></span>
                    <span class="dot yellow" aria-hidden="true"></span>
                    <span class="dot green" aria-hidden="true"></span>
                    <span class="filename">"mathematical_formulation::advection_diffusion_reaction.xml"</span>
                </div>
                <div class="card-body">
                    <div class="math-display-container">
                        <math display="block" class="native-mathml">
                            <mrow>
                                <mfrac>
                                    <mrow><mo>"∂"</mo><mi>"u"</mi></mrow>
                                    <mrow><mo>"∂"</mo><mi>"t"</mi></mrow>
                                </mfrac>
                                <mo>"="</mo>
                                <mi>"D"</mi>
                                <msup><mo>"∇"</mo><mn>"2"</mn></msup>
                                <mi>"u"</mi>
                                <mo>"+"</mo>
                                <mi>"r"</mi><mi>"u"</mi>
                                <mo>"("</mo><mn>"1"</mn><mo>"-"</mo>
                                <mfrac><mi>"u"</mi><mi>"K"</mi></mfrac>
                                <mo>")"</mo>
                            </mrow>
                        </math>
                    </div>

                    <div class="equation-terms-grid">
                        <div class="term-entry">
                            <strong class="term-sym">"∂u/∂t"</strong>
                            <span class="term-desc">"Tasa local de variación de la densidad de biomasa edáfica en el tiempo."</span>
                        </div>
                        <div class="term-entry">
                            <strong class="term-sym">"D ∇²u"</strong>
                            <span class="term-desc">"Difusión espacial isotrópica a través de la matriz de poros del suelo (humedad y temperatura)."</span>
                        </div>
                        <div class="term-entry">
                            <strong class="term-sym">"r u (1 - u/K)"</strong>
                            <span class="term-desc">"Crecimiento biológico logístico acotado por la capacidad de carga edáfica K (materia orgánica)."</span>
                        </div>
                    </div>
                </div>
            </section>

            // Sección 2: Simulador Interactivo en Wasm Puro
            <section class="simulation-section">
                <h3 class="section-title">"Simulador en Vivo: Propagación en Malla Discreta 2D"</h3>
                <p class="section-desc">
                    "Interactúa directamente en WebAssembly con el integrador numérico de diferencias finitas en tiempo real."
                </p>
                <HpcGridSimulation />
            </section>

            // Sección 3: Especificaciones de Alto Rendimiento
            <section class="hpc-specs-section">
                <SpecSheet
                    title="Motor de Cálculo AMYNTHAS-HPC"
                    subtitle="Especificaciones de microarquitectura, paralelismo numérico e integración geoespacial."
                    specs=hpc_specs
                />
            </section>
        </div>
    }
}
