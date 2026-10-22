// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.

use crate::components::chatbot::Chatbot;
use crate::pages::home::Home;
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
                        <span class="user">"oscar@bio-dev"</span><span class="colon">":"</span><span class="path">"~/portfolio"</span><span class="prompt">"$"</span> <span class="command">"./initialize_system"</span>
                    </div>
                    <h1><A href="/" class="header-link">"Oscar Mora"</A></h1>
                    <p class="subtitle">"> Conservation Technologist & Embedded Engineer"</p>
                </header>

                <main>
                    <Routes>
                        <Route path="/" view=Home />
                        <Route path="/telemetry/room" view=RoomMonitoring />
                        <Route path="/*any" view=|| view! { <div style="color: var(--accent-red);">"404 Not Found"</div> } />
                    </Routes>
                </main>

                <footer>
                    <div class="status-bar">
                        <span class="status-item">"STATUS: ONLINE"</span>
                        <span class="status-item">"STACK: RUST / BARE-METAL"</span>
                        <span class="status-item">"FRAMEWORKS: LEPTOS"</span>
                    </div>
                </footer>
            </div>
            <Chatbot />
        </Router>
    }
}
