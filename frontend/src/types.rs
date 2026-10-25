// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.

/// Estado de conexión de telemetría en tiempo real (WebSocket).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
}

impl ConnectionStatus {
    /// Representación canónica en texto del estado.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Connected => "ONLINE",
            Self::Connecting => "CONNECTING",
            Self::Disconnected => "DISCONNECTED",
        }
    }
}

/// Estado de consulta reactiva de la Dead Letter Queue (DLQ).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DlqFetchState {
    Idle,
    Loading,
    Ok,
    Err(String),
}
