// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.

// Architectural exception: Leptos heavily relies on wildcard imports for reactivity and macros,
// and `view!` macros often exceed 100 lines for complex HTML structures.
#![allow(clippy::wildcard_imports, clippy::too_many_lines, clippy::semicolon_if_nothing_returned)]

use leptos::*;

mod app;
mod components;
mod pages;
pub mod types;
pub mod utils;

use app::App;

pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}
