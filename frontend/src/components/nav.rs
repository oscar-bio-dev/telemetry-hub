// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.

use leptos::*;
use leptos_router::*;

/// Barra de navegación global estilo terminal con enlaces técnicos y estado de streaming en vivo.
#[component]
pub fn SectionNav() -> impl IntoView {
    view! {
        <nav class="terminal-nav" aria-label="Global System Navigation">
            <ul class="nav-list">
                <li class="nav-item">
                    <A href="/" class="nav-link">
                        <span class="nav-prompt">"~"</span>
                        <span class="nav-title">"/overview"</span>
                    </A>
                </li>
                <li class="nav-item">
                    <A href="/hardware" class="nav-link">
                        <span class="nav-prompt">"~"</span>
                        <span class="nav-title">"/hardware"</span>
                    </A>
                </li>
                <li class="nav-item">
                    <A href="/hpc" class="nav-link">
                        <span class="nav-prompt">"~"</span>
                        <span class="nav-title">"/hpc-modeling"</span>
                    </A>
                </li>
                <li class="nav-item">
                    <A href="/field-research" class="nav-link">
                        <span class="nav-prompt">"~"</span>
                        <span class="nav-title">"/field-research"</span>
                    </A>
                </li>
                <li class="nav-item">
                    <A href="/telemetry/room" class="nav-link nav-link-telemetry">
                        <span class="nav-prompt">"~"</span>
                        <span class="nav-title">"/telemetry/room"</span>
                        <span class="nav-live-indicator" title="Live WebSocket Telemetry Stream">
                            <span class="live-dot" aria-hidden="true"></span>
                            <span class="live-text">"LIVE"</span>
                        </span>
                    </A>
                </li>
            </ul>
        </nav>
    }
}
