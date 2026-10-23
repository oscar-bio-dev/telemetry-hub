// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Error devuelto al intentar crear un valor de telemetría con estado inválido.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum TelemetryError {
    #[error("pH value {0} is out of valid range (0.0 - 14.0)")]
    Ph(f64),
    #[error("Dissolved oxygen value {0} is out of valid range (0.0 - 40.0 mg/L)")]
    DissolvedOxygen(f64),
    #[error("Temperature value {0} is out of valid range (-50.0 - 150.0 °C)")]
    Temperature(f64),
    #[error("Humidity value {0} is out of valid range (0.0 - 100.0 %RH)")]
    Humidity(f64),
    #[error("Pressure value {0} is out of valid range (300.0 - 1100.0 hPa)")]
    Pressure(f64),
    #[error("Gas Resistance value {0} is out of valid range")]
    GasResistance(f64),
    #[error("CO2 value {0} is out of valid range (0.0 - 40000.0 ppm)")]
    Co2(f64),
    #[error("IAQ value {0} is out of valid range (0.0 - 500.0)")]
    Iaq(f64),
    #[error("PM1.0 value {0} is out of valid range (0.0 - 1000.0 µg/m³)")]
    Pm1_0(f64),
    #[error("PM2.5 value {0} is out of valid range (0.0 - 1000.0 µg/m³)")]
    Pm2_5(f64),
    #[error("PM10.0 value {0} is out of valid range (0.0 - 1000.0 µg/m³)")]
    Pm10_0(f64),
}

// ─── Newtypes with domain validation ────────────────────────────────────────

/// Representa el pH del agua o suelo. Garantizado estar entre 0.0 y 14.0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(try_from = "f64")]
pub struct Ph(f64);

impl TryFrom<f64> for Ph {
    type Error = TelemetryError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Ph {
    pub const MIN: f64 = 0.0;
    pub const MAX: f64 = 14.0;
    /// Crea una nueva instancia.
    ///
    /// # Errors
    ///
    /// Devuelve `TelemetryError` si el valor está fuera del rango permitido.
    pub fn new(value: f64) -> Result<Self, TelemetryError> {
        if value.is_nan() || !(Self::MIN..=Self::MAX).contains(&value) {
            Err(TelemetryError::Ph(value))
        } else {
            Ok(Self(value))
        }
    }
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

/// Representa el Oxígeno Disuelto (mg/L).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(try_from = "f64")]
pub struct DissolvedOxygen(f64);

impl TryFrom<f64> for DissolvedOxygen {
    type Error = TelemetryError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl DissolvedOxygen {
    pub const MIN: f64 = 0.0;
    pub const MAX: f64 = 40.0;
    /// Crea una nueva instancia.
    ///
    /// # Errors
    ///
    /// Devuelve `TelemetryError` si el valor está fuera del rango permitido.
    pub fn new(value: f64) -> Result<Self, TelemetryError> {
        if value.is_nan() || !(Self::MIN..=Self::MAX).contains(&value) {
            Err(TelemetryError::DissolvedOxygen(value))
        } else {
            Ok(Self(value))
        }
    }
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

/// Temperatura en grados Celsius.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(try_from = "f64")]
pub struct Temperature(f64);

impl TryFrom<f64> for Temperature {
    type Error = TelemetryError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Temperature {
    pub const MIN: f64 = -50.0;
    pub const MAX: f64 = 150.0;
    /// Crea una nueva instancia.
    ///
    /// # Errors
    ///
    /// Devuelve `TelemetryError` si el valor está fuera del rango permitido.
    pub fn new(value: f64) -> Result<Self, TelemetryError> {
        if value.is_nan() || !(Self::MIN..=Self::MAX).contains(&value) {
            Err(TelemetryError::Temperature(value))
        } else {
            Ok(Self(value))
        }
    }
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

/// Humedad Relativa (%).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(try_from = "f64")]
pub struct Humidity(f64);

impl TryFrom<f64> for Humidity {
    type Error = TelemetryError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Humidity {
    pub const MIN: f64 = 0.0;
    pub const MAX: f64 = 100.0;
    /// Crea una nueva instancia.
    ///
    /// # Errors
    ///
    /// Devuelve `TelemetryError` si el valor está fuera del rango permitido.
    pub fn new(value: f64) -> Result<Self, TelemetryError> {
        if value.is_nan() || !(Self::MIN..=Self::MAX).contains(&value) {
            Err(TelemetryError::Humidity(value))
        } else {
            Ok(Self(value))
        }
    }
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

/// Presión Atmosférica en hPa.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(try_from = "f64")]
pub struct Pressure(f64);

impl TryFrom<f64> for Pressure {
    type Error = TelemetryError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Pressure {
    pub const MIN: f64 = 300.0;
    pub const MAX: f64 = 1100.0;
    /// Crea una nueva instancia.
    ///
    /// # Errors
    ///
    /// Devuelve `TelemetryError` si el valor está fuera del rango permitido.
    pub fn new(value: f64) -> Result<Self, TelemetryError> {
        if value.is_nan() || !(Self::MIN..=Self::MAX).contains(&value) {
            Err(TelemetryError::Pressure(value))
        } else {
            Ok(Self(value))
        }
    }
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

/// Resistencia del gas (VOCs) en Ohmios (BME688).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(try_from = "f64")]
pub struct GasResistance(f64);

impl TryFrom<f64> for GasResistance {
    type Error = TelemetryError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl GasResistance {
    pub const MIN: f64 = 0.0;
    pub const MAX: f64 = 50_000_000.0;
    /// Crea una nueva instancia.
    ///
    /// # Errors
    ///
    /// Devuelve `TelemetryError` si el valor está fuera del rango permitido.
    pub fn new(value: f64) -> Result<Self, TelemetryError> {
        if value.is_nan() || !(Self::MIN..=Self::MAX).contains(&value) {
            Err(TelemetryError::GasResistance(value))
        } else {
            Ok(Self(value))
        }
    }
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

/// Nivel de CO2 en ppm (SCD41).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(try_from = "f64")]
pub struct Co2(f64);

impl TryFrom<f64> for Co2 {
    type Error = TelemetryError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Co2 {
    pub const MIN: f64 = 0.0;
    pub const MAX: f64 = 40_000.0;
    /// Crea una nueva instancia.
    ///
    /// # Errors
    ///
    /// Devuelve `TelemetryError` si el valor está fuera del rango permitido.
    pub fn new(value: f64) -> Result<Self, TelemetryError> {
        if value.is_nan() || !(Self::MIN..=Self::MAX).contains(&value) {
            Err(TelemetryError::Co2(value))
        } else {
            Ok(Self(value))
        }
    }
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

/// Index of Air Quality (BME688 BSEC output). Rango 0-500.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(try_from = "f64")]
pub struct Iaq(f64);

impl TryFrom<f64> for Iaq {
    type Error = TelemetryError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Iaq {
    pub const MIN: f64 = 0.0;
    pub const MAX: f64 = 500.0;
    /// Crea una nueva instancia.
    ///
    /// # Errors
    ///
    /// Devuelve `TelemetryError` si el valor está fuera del rango permitido.
    pub fn new(value: f64) -> Result<Self, TelemetryError> {
        if value.is_nan() || !(Self::MIN..=Self::MAX).contains(&value) {
            Err(TelemetryError::Iaq(value))
        } else {
            Ok(Self(value))
        }
    }
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

/// Partículas PM1.0 en µg/m³ (BMV080).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(try_from = "f64")]
pub struct Pm1_0(f64);

impl TryFrom<f64> for Pm1_0 {
    type Error = TelemetryError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Pm1_0 {
    pub const MIN: f64 = 0.0;
    pub const MAX: f64 = 1000.0;
    /// Crea una nueva instancia.
    ///
    /// # Errors
    ///
    /// Devuelve `TelemetryError` si el valor está fuera del rango permitido.
    pub fn new(value: f64) -> Result<Self, TelemetryError> {
        if value.is_nan() || !(Self::MIN..=Self::MAX).contains(&value) {
            Err(TelemetryError::Pm1_0(value))
        } else {
            Ok(Self(value))
        }
    }
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

/// Partículas PM2.5 en µg/m³ (BMV080).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(try_from = "f64")]
pub struct Pm2_5(f64);

impl TryFrom<f64> for Pm2_5 {
    type Error = TelemetryError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Pm2_5 {
    pub const MIN: f64 = 0.0;
    pub const MAX: f64 = 1000.0;
    /// Crea una nueva instancia.
    ///
    /// # Errors
    ///
    /// Devuelve `TelemetryError` si el valor está fuera del rango permitido.
    pub fn new(value: f64) -> Result<Self, TelemetryError> {
        if value.is_nan() || !(Self::MIN..=Self::MAX).contains(&value) {
            Err(TelemetryError::Pm2_5(value))
        } else {
            Ok(Self(value))
        }
    }
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

/// Partículas PM10.0 en µg/m³ (BMV080).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(try_from = "f64")]
pub struct Pm10_0(f64);

impl TryFrom<f64> for Pm10_0 {
    type Error = TelemetryError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Pm10_0 {
    pub const MIN: f64 = 0.0;
    pub const MAX: f64 = 1000.0;
    /// Crea una nueva instancia.
    ///
    /// # Errors
    ///
    /// Devuelve `TelemetryError` si el valor está fuera del rango permitido.
    pub fn new(value: f64) -> Result<Self, TelemetryError> {
        if value.is_nan() || !(Self::MIN..=Self::MAX).contains(&value) {
            Err(TelemetryError::Pm10_0(value))
        } else {
            Ok(Self(value))
        }
    }
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

// ─── Regex para validación de device_id ─────────────────────────────────────

lazy_static::lazy_static! {
    static ref DEVICE_ID_REGEX: regex::Regex = regex::Regex::new(r"^[a-zA-Z0-9_:.-]+$").unwrap();
}

// ─── Payload de telemetría (Modelo de Dominio) ──────────────────────────────

/// Payload completo de telemetría proveniente del hardware edge (ESP32, RP2350).
/// Alineado con el Mega-Schema canónico del Gateway.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema, Validate)]
pub struct TelemetryPayload {
    /// Versión del protocolo
    pub protocol_version: u32,
    /// Versión del esquema
    pub schema_version: u32,

    /// ID único del evento (UUID) para idempotencia
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    #[validate(length(min = 1, max = 64))]
    pub event_id: String,

    /// ID del gateway (MAC del ESP32-P4)
    #[schema(example = "gateway-AA:BB:CC:DD:EE:FF")]
    #[validate(length(min = 1, max = 64))]
    pub gateway_id: String,

    /// ID único del dispositivo `IoT` (ej. `"sensor-AA:BB:CC:DD:EE:FF"`)
    #[schema(example = "sensor-AA:BB:CC:DD:EE:FF")]
    #[validate(
        length(min = 1, max = 64, message = "El device_id debe tener entre 1 y 64 caracteres"),
        regex(path = *DEVICE_ID_REGEX, message = "El device_id solo puede contener letras, números, guiones, guiones bajos, puntos y dos puntos")
    )]
    pub device_id: String,

    /// Secuencia originada por el nodo
    pub node_sequence: u32,

    /// Timestamp de lectura en el nodo (UTC)
    #[schema(example = "2026-09-04T10:00:00Z")]
    pub measured_at: DateTime<Utc>,

    /// Timestamp de ingestión en el gateway (UTC)
    #[schema(example = "2026-09-04T10:00:01Z")]
    pub ingested_at: DateTime<Utc>,

    // ─── Ambiental y Calidad de Aire (BME688 / SCD41) ───────────────────
    /// Temperatura validada en grados Celsius
    pub temperature: Option<Temperature>,
    /// Humedad Relativa (%)
    pub humidity: Option<Humidity>,
    /// Presión atmosférica (hPa)
    pub pressure: Option<Pressure>,
    /// Resistencia del gas / VOCs (Ohmios)
    pub gas_resistance: Option<GasResistance>,
    /// Index of Air Quality (BME688 BSEC)
    pub iaq: Option<Iaq>,
    /// CO2 en ppm (SCD41)
    pub co2: Option<Co2>,

    // ─── Partículas (BMV080) ────────────────────────────────────────────
    /// Partículas PM1.0 (µg/m³)
    pub pm1_0: Option<Pm1_0>,
    /// Partículas PM2.5 (µg/m³)
    pub pm2_5: Option<Pm2_5>,
    /// Partículas PM10.0 (µg/m³)
    pub pm10_0: Option<Pm10_0>,

    // ─── Agua y Suelo ───────────────────────────────────────────────────
    /// pH del agua o suelo (0.0 - 14.0)
    pub ph: Option<Ph>,
    /// Oxígeno disuelto (mg/L)
    pub dissolved_oxygen: Option<DissolvedOxygen>,

    // ─── Diagnósticos del Nodo ──────────────────────────────────────────
    /// Voltaje de batería en mV
    pub battery_mv: Option<u32>,
    /// Contador de ciclos de deep sleep
    pub sleep_cycles: Option<u32>,
    /// Warmup BSEC en progreso (12 pulsos iniciales)
    pub is_calibrating: Option<bool>,

    // ─── Sensores Virtuales BSEC ────────────────────────────────────────
    /// CO2 equivalente derivado de VOCs en ppm
    pub eco2: Option<f64>,
    /// Breath VOC equivalente en ppm
    pub bvoc: Option<f64>,
    /// Total VOC equivalente en ppb
    pub tvoc: Option<f64>,

    // ─── Partículas BMV080 — Conteo Numérico ────────────────────────────
    /// Concentración numérica PM1.0 (#/m³)
    pub pm1_0_count: Option<f64>,
    /// Concentración numérica PM2.5 (#/m³)
    pub pm2_5_count: Option<f64>,
    /// Concentración numérica PM10.0 (#/m³)
    pub pm10_0_count: Option<f64>,

    // ─── Salud del Sensor Láser BMV080 ──────────────────────────────────
    /// Indica si la óptica o lente del sensor láser está sucia u obstruida
    pub is_laser_obstructed: Option<bool>,
    /// Indica si la concentración supera el rango máximo de medición
    pub is_pm_out_of_range: Option<bool>,
    /// Tiempo de operación acumulado del láser en segundos
    pub laser_runtime: Option<f64>,

    // ─── Diagnósticos Pasivos y Estado ──────────────────────────────────
    /// Bitmask de errores de hardware activos
    pub system_error_bitmask: Option<u32>,
    /// Estado operativo del nodo
    pub status: Option<NodeStatus>,
}

// ─── Enums Canónicos del Mega-Esquema ──────────────────────────────────────

/// Estado operativo reportado por el nodo sensor.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
    prost::Enumeration,
)]
#[repr(i32)]
pub enum NodeStatus {
    /// Nodo operando y muestreando normalmente en producción
    Monitoring = 0,
    /// Nodo en fase inicial de estabilización y warmup de sensores (12 pulsos)
    Calibrating = 1,
    /// Nodo ejecutando autoprueba de silicio
    SelfTesting = 2,
    /// Fallo crítico de hardware detectado
    HardwareError = 3,
}

impl NodeStatus {
    /// Devuelve el identificador de texto canónico del estado operativo.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Monitoring => "MONITORING",
            Self::Calibrating => "CALIBRATING",
            Self::SelfTesting => "SELF_TESTING",
            Self::HardwareError => "HARDWARE_ERROR",
        }
    }
}

impl std::fmt::Display for NodeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Comandos que pueden ser enviados hacia el nodo (Downlink).
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
    prost::Enumeration,
)]
#[allow(clippy::enum_variant_names)]
#[repr(i32)]
pub enum Command {
    /// Ningún comando pendiente
    CmdNone = 0,
    /// Ejecutar autodiagnóstico I2C/ASIC
    CmdRunSelfTest = 1,
    /// Reiniciar microcontrolador
    CmdReboot = 2,
}

// ─── DTO Protobuf (Wire Format — f32 del hardware) ──────────────────────────

/// DTO binario Protobuf alineado con el Mega-Schema del Gateway.
/// Los campos usan `float` (f32) para reflejar el contrato de cable 1:1.
#[derive(Clone, PartialEq, prost::Message)]
pub struct TelemetryPayloadPb {
    #[prost(uint32, tag = "1")]
    pub protocol_version: u32,
    #[prost(uint32, tag = "2")]
    pub schema_version: u32,
    #[prost(string, tag = "3")]
    pub event_id: String,
    #[prost(string, tag = "4")]
    pub gateway_id: String,
    #[prost(string, tag = "5")]
    pub device_id: String,
    #[prost(uint32, tag = "6")]
    pub node_sequence: u32,
    #[prost(uint64, tag = "7")]
    pub measured_at_ms: u64,
    #[prost(uint64, tag = "8")]
    pub ingested_at_ms: u64,

    // Ambiental (BME688 / SCD41) — f32 wire types
    #[prost(float, optional, tag = "9")]
    pub temperature: Option<f32>,
    #[prost(float, optional, tag = "10")]
    pub humidity: Option<f32>,
    #[prost(float, optional, tag = "11")]
    pub pressure: Option<f32>,
    #[prost(float, optional, tag = "12")]
    pub gas_resistance: Option<f32>,
    #[prost(float, optional, tag = "13")]
    pub iaq: Option<f32>,
    #[prost(uint32, optional, tag = "14")]
    pub co2: Option<u32>,

    // Partículas (BMV080)
    #[prost(float, optional, tag = "15")]
    pub pm1_0: Option<f32>,
    #[prost(float, optional, tag = "16")]
    pub pm2_5: Option<f32>,
    #[prost(float, optional, tag = "17")]
    pub pm10_0: Option<f32>,

    // Agua y Suelo
    #[prost(float, optional, tag = "18")]
    pub ph: Option<f32>,
    #[prost(float, optional, tag = "19")]
    pub dissolved_oxygen: Option<f32>,

    // Diagnósticos del Nodo
    #[prost(uint32, optional, tag = "20")]
    pub battery_mv: Option<u32>,
    #[prost(uint32, optional, tag = "21")]
    pub sleep_cycles: Option<u32>,
    #[prost(bool, optional, tag = "22")]
    pub is_calibrating: Option<bool>,

    // VOCs y CO2 Equivalente (BSEC)
    #[prost(float, optional, tag = "23")]
    pub eco2: Option<f32>,
    #[prost(float, optional, tag = "24")]
    pub bvoc: Option<f32>,
    #[prost(float, optional, tag = "25")]
    pub tvoc: Option<f32>,

    // Partículas (BMV080 Láser) — Conteo Numérico (#/m³)
    #[prost(float, optional, tag = "26")]
    pub pm1_0_count: Option<f32>,
    #[prost(float, optional, tag = "27")]
    pub pm2_5_count: Option<f32>,
    #[prost(float, optional, tag = "28")]
    pub pm10_0_count: Option<f32>,

    // Estado Láser BMV080
    #[prost(bool, optional, tag = "29")]
    pub is_laser_obstructed: Option<bool>,
    #[prost(bool, optional, tag = "30")]
    pub is_pm_out_of_range: Option<bool>,
    #[prost(float, optional, tag = "31")]
    pub laser_runtime: Option<f32>,

    // Status Report Pasivo del Nodo
    #[prost(uint32, optional, tag = "32")]
    pub system_error_bitmask: Option<u32>,
    #[prost(enumeration = "NodeStatus", optional, tag = "33")]
    pub status: Option<i32>,
}

/// DTO Protobuf para ACK transmitido por el Gateway hacia el Nodo.
#[derive(Clone, PartialEq, Eq, prost::Message)]
pub struct GatewayAckPb {
    #[prost(enumeration = "Command", optional, tag = "1")]
    pub command: Option<i32>,
    #[prost(uint64, optional, tag = "2")]
    pub current_epoch_s: Option<u64>,
}

/// DTO Protobuf para reporte de autodiagnóstico activo emitido por el nodo.
#[derive(Clone, PartialEq, Eq, prost::Message)]
pub struct DiagnosticReportPb {
    #[prost(uint32, optional, tag = "1")]
    pub system_error_bitmask: Option<u32>,
    #[prost(bool, optional, tag = "2")]
    pub scd41_passed: Option<bool>,
    #[prost(bool, optional, tag = "3")]
    pub bmv080_passed: Option<bool>,
    #[prost(bool, optional, tag = "4")]
    pub bme688_passed: Option<bool>,
}

/// Conversión del DTO Protobuf (f32 wire) al Modelo de Dominio (f64 analítico).
/// Cast explícito `f32 as f64` — widening sin pérdida.
impl TryFrom<TelemetryPayloadPb> for TelemetryPayload {
    type Error = TelemetryError;
    fn try_from(pb: TelemetryPayloadPb) -> Result<Self, Self::Error> {
        let ingested_at = if pb.ingested_at_ms == 0 {
            Utc::now()
        } else {
            chrono::DateTime::from_timestamp_millis(i64::try_from(pb.ingested_at_ms).unwrap_or(0))
                .unwrap_or_else(Utc::now)
        };

        // Si el nodo perdió la batería RTC o no tiene sincronización (measured_at_ms == 0), asignar Utc::now()
        let measured_at = if pb.measured_at_ms == 0 {
            Utc::now()
        } else {
            chrono::DateTime::from_timestamp_millis(i64::try_from(pb.measured_at_ms).unwrap_or(0))
                .unwrap_or_else(Utc::now)
        };

        let status = pb.status.and_then(|s| NodeStatus::try_from(s).ok());

        Ok(Self {
            protocol_version: pb.protocol_version,
            schema_version: pb.schema_version,
            event_id: pb.event_id,
            gateway_id: pb.gateway_id,
            device_id: pb.device_id,
            node_sequence: pb.node_sequence,
            measured_at,
            ingested_at,
            temperature: pb.temperature.map(|v| Temperature::new(f64::from(v))).transpose()?,
            humidity: pb.humidity.map(|v| Humidity::new(f64::from(v))).transpose()?,
            pressure: pb.pressure.map(|v| Pressure::new(f64::from(v))).transpose()?,
            gas_resistance: pb
                .gas_resistance
                .map(|v| GasResistance::new(f64::from(v)))
                .transpose()?,
            iaq: pb.iaq.map(|v| Iaq::new(f64::from(v))).transpose()?,
            co2: pb.co2.map(|v| Co2::new(f64::from(v))).transpose()?,
            pm1_0: pb.pm1_0.map(|v| Pm1_0::new(f64::from(v))).transpose()?,
            pm2_5: pb.pm2_5.map(|v| Pm2_5::new(f64::from(v))).transpose()?,
            pm10_0: pb.pm10_0.map(|v| Pm10_0::new(f64::from(v))).transpose()?,
            ph: pb.ph.map(|v| Ph::new(f64::from(v))).transpose()?,
            dissolved_oxygen: pb
                .dissolved_oxygen
                .map(|v| DissolvedOxygen::new(f64::from(v)))
                .transpose()?,
            battery_mv: pb.battery_mv,
            sleep_cycles: pb.sleep_cycles,
            is_calibrating: pb.is_calibrating,
            eco2: pb.eco2.map(f64::from),
            bvoc: pb.bvoc.map(f64::from),
            tvoc: pb.tvoc.map(f64::from),
            pm1_0_count: pb.pm1_0_count.map(f64::from),
            pm2_5_count: pb.pm2_5_count.map(f64::from),
            pm10_0_count: pb.pm10_0_count.map(f64::from),
            is_laser_obstructed: pb.is_laser_obstructed,
            is_pm_out_of_range: pb.is_pm_out_of_range,
            laser_runtime: pb.laser_runtime.map(f64::from),
            system_error_bitmask: pb.system_error_bitmask,
            status,
        })
    }
}

// ─── Gateway Health Event (Diagnósticos del Edge Gateway) ───────────────────

/// Evento de salud del Edge Telemetry Gateway (ESP32-P4).
/// Publicado independientemente de la telemetría de los nodos.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema, Validate)]
pub struct GatewayHealthEvent {
    /// ID del gateway (MAC del ESP32-P4)
    #[schema(example = "gateway-AA:BB:CC:DD:EE:FF")]
    #[validate(length(
        min = 1,
        max = 64,
        message = "El gateway_id debe tener entre 1 y 64 caracteres"
    ))]
    pub gateway_id: String,

    /// Timestamp del evento (UTC)
    pub timestamp: DateTime<Utc>,

    /// True si la `MicroSD` ha fallado o fue extraída
    pub is_degraded_mode: bool,

    /// Estado físico del montaje VFS de la `MicroSD`
    pub sd_card_mounted: bool,
    /// Capacidad total de la `MicroSD` en MB
    pub sd_card_total_mb: Option<u32>,
    /// Espacio libre de la `MicroSD` en MB
    pub sd_card_free_mb: Option<u32>,
    /// Contador acumulado de errores I/O
    pub sd_io_errors: u32,

    /// Tiempo de actividad en segundos
    pub uptime_seconds: u32,
    /// Heap libre en bytes
    pub free_heap_bytes: u32,

    /// Mensaje de alerta crítica para operadores
    pub alert_message: Option<String>,
}

/// DTO Protobuf para el evento de salud del Gateway.
#[derive(Clone, PartialEq, Eq, prost::Message)]
pub struct GatewayHealthEventPb {
    #[prost(string, tag = "1")]
    pub gateway_id: String,
    #[prost(bool, tag = "2")]
    pub is_degraded_mode: bool,
    #[prost(bool, tag = "3")]
    pub sd_card_mounted: bool,
    #[prost(uint32, tag = "4")]
    pub sd_card_total_mb: u32,
    #[prost(uint32, tag = "5")]
    pub sd_card_free_mb: u32,
    #[prost(uint32, tag = "6")]
    pub sd_io_errors: u32,
    #[prost(uint32, tag = "7")]
    pub uptime_seconds: u32,
    #[prost(uint32, tag = "8")]
    pub free_heap_bytes: u32,
    #[prost(string, tag = "9")]
    pub alert_message: String,
}

impl From<GatewayHealthEventPb> for GatewayHealthEvent {
    fn from(pb: GatewayHealthEventPb) -> Self {
        Self {
            gateway_id: pb.gateway_id,
            timestamp: Utc::now(),
            is_degraded_mode: pb.is_degraded_mode,
            sd_card_mounted: pb.sd_card_mounted,
            sd_card_total_mb: if pb.sd_card_total_mb > 0 {
                Some(pb.sd_card_total_mb)
            } else {
                None
            },
            sd_card_free_mb: if pb.sd_card_free_mb > 0 { Some(pb.sd_card_free_mb) } else { None },
            sd_io_errors: pb.sd_io_errors,
            uptime_seconds: pb.uptime_seconds,
            free_heap_bytes: pb.free_heap_bytes,
            alert_message: if pb.alert_message.is_empty() { None } else { Some(pb.alert_message) },
        }
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_ph_validation(val in -1.0..15.0f64) {
            let ph = Ph::new(val);
            if (0.0..=14.0).contains(&val) {
                assert!(ph.is_ok());
            } else {
                assert!(ph.is_err());
            }
        }

        #[test]
        fn test_temperature_validation(val in -60.0..160.0f64) {
            let temp = Temperature::new(val);
            if (-50.0..=150.0).contains(&val) {
                assert!(temp.is_ok());
            } else {
                assert!(temp.is_err());
            }
        }

        #[test]
        fn test_iaq_validation(val in -10.0..510.0f64) {
            let iaq = Iaq::new(val);
            if (0.0..=500.0).contains(&val) {
                assert!(iaq.is_ok());
            } else {
                assert!(iaq.is_err());
            }
        }

        #[test]
        fn test_pm2_5_validation(val in -10.0..1010.0f64) {
            let pm = Pm2_5::new(val);
            if (0.0..=1000.0).contains(&val) {
                assert!(pm.is_ok());
            } else {
                assert!(pm.is_err());
            }
        }

        #[test]
        fn test_telemetry_protobuf_conversion(
            temp in -50.0..150.0f32,
            hum in 0.0..100.0f32
        ) {
            let pb = TelemetryPayloadPb {
                protocol_version: 1,
                schema_version: 1,
                event_id: "evt-123".to_string(),
                gateway_id: "gw-1".to_string(),
                device_id: "test-node".to_string(),
                node_sequence: 1,
                measured_at_ms: 1_700_000_000_000,
                ingested_at_ms: 1_700_000_000_100,
                temperature: Some(temp),
                humidity: Some(hum),
                pressure: None,
                gas_resistance: None,
                iaq: None,
                co2: None,
                pm1_0: None,
                pm2_5: None,
                pm10_0: None,
                ph: None,
                dissolved_oxygen: None,
                battery_mv: Some(3300),
                sleep_cycles: Some(42),
                is_calibrating: Some(false),
                eco2: Some(400.0),
                bvoc: Some(0.1),
                tvoc: Some(15.0),
                pm1_0_count: Some(120.0),
                pm2_5_count: Some(50.0),
                pm10_0_count: Some(10.0),
                is_laser_obstructed: Some(false),
                is_pm_out_of_range: Some(false),
                laser_runtime: Some(1.2),
                system_error_bitmask: Some(0),
                status: Some(NodeStatus::Monitoring as i32),
            };

            let payload: Result<TelemetryPayload, _> = pb.try_into();
            assert!(payload.is_ok());
            let p = payload.unwrap();
            assert_eq!(p.battery_mv, Some(3300));
            assert_eq!(p.sleep_cycles, Some(42));
            assert_eq!(p.event_id, "evt-123");
            assert_eq!(p.status, Some(NodeStatus::Monitoring));
            assert_eq!(p.is_calibrating, Some(false));
            assert_eq!(p.is_laser_obstructed, Some(false));
        }
    }

    #[test]
    fn test_telemetry_timestamp_zero_fallback() {
        let pb = TelemetryPayloadPb {
            protocol_version: 1,
            schema_version: 1,
            event_id: "evt-rtc-zero".to_string(),
            gateway_id: "gw-1".to_string(),
            device_id: "node-rtc-zero".to_string(),
            node_sequence: 1,
            measured_at_ms: 0,
            ingested_at_ms: 1_700_000_000_000,
            temperature: Some(25.0),
            humidity: Some(50.0),
            pressure: None,
            gas_resistance: None,
            iaq: None,
            co2: None,
            pm1_0: None,
            pm2_5: None,
            pm10_0: None,
            ph: None,
            dissolved_oxygen: None,
            battery_mv: None,
            sleep_cycles: None,
            is_calibrating: None,
            eco2: None,
            bvoc: None,
            tvoc: None,
            pm1_0_count: None,
            pm2_5_count: None,
            pm10_0_count: None,
            is_laser_obstructed: None,
            is_pm_out_of_range: None,
            laser_runtime: None,
            system_error_bitmask: None,
            status: None,
        };

        let before = Utc::now();
        let payload: TelemetryPayload = pb.try_into().expect("conversion should succeed");
        let after = Utc::now();
        assert!(payload.measured_at >= before && payload.measured_at <= after);
        assert_eq!(payload.ingested_at.timestamp_millis(), 1_700_000_000_000);
    }

    #[test]
    fn test_invalid_json_deserialization() {
        let json =
            r#"{"device_id":"node-1","measured_at":"2026-08-30T10:00:00Z","temperature": 999.0}"#;
        let result: Result<TelemetryPayload, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_json_deserialization_minimal() {
        // Now requires protocol_version, schema_version, event_id, gateway_id, node_sequence, measured_at, ingested_at
        let json = r#"{
            "protocol_version": 1,
            "schema_version": 1,
            "event_id": "evt-123",
            "gateway_id": "gw-1",
            "device_id": "node-1",
            "node_sequence": 1,
            "measured_at": "2026-09-04T10:00:00Z",
            "ingested_at": "2026-09-04T10:00:00Z"
        }"#;
        let result: Result<TelemetryPayload, _> = serde_json::from_str(json);
        assert!(result.is_ok());
        let p = result.unwrap();
        assert!(p.temperature.is_none());
        assert!(p.co2.is_none());
        assert!(p.pm2_5.is_none());
    }

    #[test]
    fn test_valid_json_deserialization_full() {
        let json = r#"{
            "protocol_version": 1,
            "schema_version": 1,
            "event_id": "evt-123",
            "gateway_id": "gw-1",
            "device_id":"sensor-AA:BB:CC:DD:EE:FF",
            "node_sequence": 1,
            "measured_at":"2026-09-04T10:00:00Z",
            "ingested_at":"2026-09-04T10:00:01Z",
            "temperature": 25.5,
            "humidity": 60.0,
            "pressure": 1013.25,
            "iaq": 50.0,
            "co2": 420.0,
            "pm2_5": 12.5,
            "battery_mv": 3300
        }"#;
        let result: Result<TelemetryPayload, _> = serde_json::from_str(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_gateway_health_event_from_pb() {
        let pb = GatewayHealthEventPb {
            gateway_id: "gateway-AA:BB:CC:DD:EE:FF".to_string(),
            is_degraded_mode: true,
            sd_card_mounted: false,
            sd_card_total_mb: 32768,
            sd_card_free_mb: 0,
            sd_io_errors: 42,
            uptime_seconds: 3600,
            free_heap_bytes: 200_000,
            alert_message: "SD card removed".to_string(),
        };

        let event: GatewayHealthEvent = pb.into();
        assert_eq!(event.gateway_id, "gateway-AA:BB:CC:DD:EE:FF");
        assert!(event.is_degraded_mode);
        assert!(!event.sd_card_mounted);
        assert_eq!(event.sd_io_errors, 42);
        assert_eq!(event.alert_message.as_deref(), Some("SD card removed"));
    }

    #[test]
    fn test_node_status_as_str() {
        assert_eq!(NodeStatus::Monitoring.as_str(), "MONITORING");
        assert_eq!(NodeStatus::Calibrating.as_str(), "CALIBRATING");
        assert_eq!(NodeStatus::SelfTesting.as_str(), "SELF_TESTING");
        assert_eq!(NodeStatus::HardwareError.as_str(), "HARDWARE_ERROR");
        assert_eq!(format!("{}", NodeStatus::Monitoring), "MONITORING");
    }
}
