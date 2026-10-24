// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.

use crate::components::chatbot::Chatbot;
use crate::components::nav::SectionNav;
use crate::pages::field_research::FieldResearchPage;
use crate::pages::hardware::HardwarePage;
use crate::pages::home::Home;
use crate::pages::hpc::HpcPage;
use crate::pages::room_monitoring::RoomMonitoring;
use leptos::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <div class="container">
                <header class="hero-header">
                    <div class="prompt-line">
                        <span class="user">"oscar@bio-dev"</span>
                        <span class="colon">":"</span>
                        <span class="path">"~/telemetry-hub"</span>
                        <span class="prompt">"$"</span>
                        <span class="command">"./initialize_scada_hmi"</span>
                        <span class="cursor-blink" aria-hidden="true"></span>
                    </div>
                    <h1><A href="/" class="header-link">"Oscar Mora"</A></h1>
                    <p class="subtitle">"> Conservation Technologist & Embedded Systems Engineer | EcoWest Instrument"</p>
                </header>

                <SectionNav />

                <main class="main-content">
                    <Routes>
                        <Route path="/" view=Home />
                        <Route path="/hardware" view=HardwarePage />
                        <Route path="/hpc" view=HpcPage />
                        <Route path="/field-research" view=FieldResearchPage />
                        <Route path="/telemetry/room" view=RoomMonitoring />
                        <Route path="/*any" view=|| view! {
                            <div class="terminal-card not-found-card">
                                <div class="card-header">
                                    <span class="dot red" aria-hidden="true"></span>
                                    <span class="filename">"404_not_found.err"</span>
                                </div>
                                <div class="card-body">
                                    <p class="error-text">"[ERROR 404] Ruta no encontrada en el sistema HMI."</p>
                                    <A href="/" class="nav-link">"← Regresar a ~/overview"</A>
                                </div>
                            </div>
                        } />
                    </Routes>
                </main>

                <footer>
                    <div class="status-bar">
                        <span class="status-item">"● STATUS: ONLINE"</span>
                        <span class="status-item">{format!("VERSION: v{}", env!("CARGO_PKG_VERSION"))}</span>
                        <span class="status-item">"TARGET: wasm32-unknown-unknown"</span>
                        <span class="status-item">"ARCH: Gruvbox SCADA HMI"</span>
                    </div>
                </footer>
            </div>
            <Chatbot />
        </Router>
    }
}
