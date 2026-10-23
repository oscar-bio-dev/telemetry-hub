-- Migration 0006: Add Canonical Mega-Schema v2.0 fields (fields 22..33)
-- Covers BSEC warmup, virtual sensors (eCO2, bVOC, TVOC), BMV080 particulate counts,
-- laser diagnostics (optical obstruction, out of range, runtime), and node status report.

-- Field 22: Warmup BSEC en progreso (12 pulsos iniciales)
ALTER TABLE telemetry ADD COLUMN is_calibrating BOOLEAN;

-- Fields 23-25: Sensores Virtuales BSEC
ALTER TABLE telemetry ADD COLUMN eco2 DOUBLE PRECISION;
ALTER TABLE telemetry ADD COLUMN bvoc DOUBLE PRECISION;
ALTER TABLE telemetry ADD COLUMN tvoc DOUBLE PRECISION;

-- Fields 26-28: Partículas BMV080 (Concentración Numérica #/m³)
ALTER TABLE telemetry ADD COLUMN pm1_0_count DOUBLE PRECISION;
ALTER TABLE telemetry ADD COLUMN pm2_5_count DOUBLE PRECISION;
ALTER TABLE telemetry ADD COLUMN pm10_0_count DOUBLE PRECISION;

-- Fields 29-31: Diagnósticos Láser BMV080
ALTER TABLE telemetry ADD COLUMN is_laser_obstructed BOOLEAN;
ALTER TABLE telemetry ADD COLUMN is_pm_out_of_range BOOLEAN;
ALTER TABLE telemetry ADD COLUMN laser_runtime DOUBLE PRECISION;

-- Fields 32-33: Diagnósticos y Estado Operativo del Nodo
ALTER TABLE telemetry ADD COLUMN system_error_bitmask BIGINT;
ALTER TABLE telemetry ADD COLUMN status VARCHAR(32);
