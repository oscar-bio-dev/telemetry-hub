// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.
//
// This file is part of the oscar-bio-dev platform.
// Unauthorized copying of this file, via any medium, is strictly prohibited.
// Proprietary and confidential.

/// Submódulo encargado de la autenticación.
pub mod auth;
/// Submódulo encargado del Chatbot LLM impulsado por Gemini.
pub mod chat;
/// Submódulo encargado de la lectura global del gemelo digital.
pub mod digital_twin;
/// Submódulo encargado de la Dead Letter Queue (DLQ).
pub mod dlq;
/// Submódulo de Kubernetes Probes
pub mod health;
/// Submódulo encargado del streaming y control en tiempo real.
pub mod ws;
