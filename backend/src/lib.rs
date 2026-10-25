// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.
//
// This file is part of the telemetry-hub platform.
// Unauthorized copying of this file, via any medium, is strictly prohibited.
// Proprietary and confidential.

#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
    unused,
    missing_docs
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::multiple_crate_versions
)]

//! `telemetry_hub` es el núcleo de telemetría y gemelo digital.
//!
//! Esta biblioteca provee los endpoints y modelos de dominio necesarios
//! para la ingesta de datos ambientales provenientes de hardware `IoT`.

/// Módulo de controladores (handlers) y rutas HTTP de la API.
pub mod api;
/// Módulo de lógica de negocio y tipos de datos fuertes (Clean Architecture).
pub mod domain;
/// Módulo de infraestructura y persistencia de datos.
pub mod infrastructure;
