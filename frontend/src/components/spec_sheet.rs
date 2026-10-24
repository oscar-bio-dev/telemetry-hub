// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.

use leptos::*;

/// Fila individual de especificación técnica para instrumentos de grado industrial.
#[derive(Clone, Debug, PartialEq)]
pub struct SpecRow {
    pub parameter: &'static str,
    pub value: &'static str,
    pub unit: &'static str,
    pub tolerance: &'static str,
    pub notes: &'static str,
}

impl SpecRow {
    #[must_use]
    pub const fn new(
        parameter: &'static str,
        value: &'static str,
        unit: &'static str,
        tolerance: &'static str,
        notes: &'static str,
    ) -> Self {
        Self { parameter, value, unit, tolerance, notes }
    }
}

/// Componente reutilizable para renderizar hojas de especificaciones técnicas (datasheets)
/// con tipografía monoespaciada y alineación numérica tabular.
#[component]
pub fn SpecSheet(
    title: &'static str,
    specs: Vec<SpecRow>,
    #[prop(optional)] subtitle: Option<&'static str>,
) -> impl IntoView {
    view! {
        <section class="terminal-card spec-sheet-card">
            <div class="card-header spec-sheet-header">
                <span class="dot red" aria-hidden="true"></span>
                <span class="dot yellow" aria-hidden="true"></span>
                <span class="dot green" aria-hidden="true"></span>
                <span class="filename">"datasheet::"<span class="spec-sheet-title">{title}</span>".spec"</span>
            </div>
            <div class="card-body spec-sheet-body">
                {subtitle.map(|sub| view! { <p class="spec-sheet-subtitle">{sub}</p> })}
                <div class="spec-table-wrapper">
                    <table class="specs-table tabular-nums">
                        <thead>
                            <tr>
                                <th scope="col">"Parámetro"</th>
                                <th scope="col">"Valor Nominal"</th>
                                <th scope="col">"Unidad"</th>
                                <th scope="col">"Tolerancia / Rango"</th>
                                <th scope="col">"Notas / Estándar"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {specs.into_iter().map(|row| view! {
                                <tr class="spec-row">
                                    <td class="spec-param">{row.parameter}</td>
                                    <td class="spec-value"><strong>{row.value}</strong></td>
                                    <td class="spec-unit">{row.unit}</td>
                                    <td class="spec-tolerance">{row.tolerance}</td>
                                    <td class="spec-notes">{row.notes}</td>
                                </tr>
                            }).collect_view()}
                        </tbody>
                    </table>
                </div>
            </div>
        </section>
    }
}
