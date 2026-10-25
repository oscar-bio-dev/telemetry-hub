// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.

use crate::types::ConnectionStatus;
use leptos::*;
use leptos_router::*;

/// Barra de navegación global estilo terminal con enlaces técnicos y estado de streaming en vivo.
#[component]
pub fn SectionNav() -> impl IntoView {
    let conn_status = use_context::<RwSignal<ConnectionStatus>>()
        .unwrap_or_else(|| create_rw_signal(ConnectionStatus::Disconnected));

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
                            {move || match conn_status.get() {
                                ConnectionStatus::Connected => view! {
                                    <span class="live-dot live-dot-connected" aria-hidden="true"></span>
                                    <span class="live-text live-text-connected">"LIVE"</span>
                                }.into_view(),
                                ConnectionStatus::Connecting => view! {
                                    <span class="live-dot live-dot-connecting" aria-hidden="true"></span>
                                    <span class="live-text live-text-connecting">"LINK..."</span>
                                }.into_view(),
                                ConnectionStatus::Disconnected => view! {
                                    <span class="live-dot live-dot-disconnected" aria-hidden="true"></span>
                                    <span class="live-text live-text-disconnected">"OFFLINE"</span>
                                }.into_view(),
                            }}
                        </span>
                    </A>
                </li>
            </ul>
        </nav>
    }
}
