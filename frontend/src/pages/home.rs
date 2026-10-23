// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.

use leptos::*;
use leptos_router::*;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <div>
            <section id="about" class="terminal-card" style="margin-bottom: 2rem;">
                <div class="card-header">
                    <span class="dot red" aria-hidden="true"></span>
                    <span class="dot yellow" aria-hidden="true"></span>
                    <span class="dot green" aria-hidden="true"></span>
                    <span class="filename">"about_me.md"</span>
                </div>
                <div class="card-body">
                    <p>"Translating biological complexity into ultra-low-power hardware solutions for ecological monitoring."</p>
                    <p>"Based in Pueblo West, Colorado. Bridging the gap between biology and bare-metal engineering using Pure Rust and embedded C/C++."</p>
                </div>
            </section>

            <section id="projects">
                <h2 class="section-title">"Telemetry & Systems"</h2>
                <div class="project-grid">
                    <a href="https://github.com/oscar-bio-dev/telemetry-hub" target="_blank" class="project-item">
                        <span class="project-icon" aria-hidden="true">"🦀"</span>
                        <div class="project-info">
                            <h3>"telemetry-hub"</h3>
                            <p>"Space-grade IoT telemetry platform built in Rust (Axum + Tokio)."</p>
                        </div>
                    </a>
                    <article class="project-item">
                        <span class="project-icon" aria-hidden="true">"💧"</span>
                        <div class="project-info">
                            <h3>"Aquaculture DO Meter"</h3>
                            <p>"Bare-metal C on TI MSPM0 for high-precision dissolved oxygen sensing."</p>
                        </div>
                    </article>
                    <article class="project-item">
                        <span class="project-icon" aria-hidden="true">"🌱"</span>
                        <div class="project-info">
                            <h3>"Smart Soil Node"</h3>
                            <p>"RP2350 industrial RS485 controller with a native Qt dashboard."</p>
                        </div>
                    </article>
                    <A href="/telemetry/room" class="project-item">
                        <span class="project-icon" aria-hidden="true">"🏠"</span>
                        <div class="project-info">
                            <h3 style="color: var(--accent-green);">"Room Monitoring"</h3>
                            <p>"Esp32 node for indoor air quality. [ ONLINE / STREAMING ]"</p>
                        </div>
                    </A>
                </div>
            </section>
        </div>
    }
}
