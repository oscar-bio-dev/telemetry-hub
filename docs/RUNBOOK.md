# 🛰️ EcoWest Instrument — Operational Runbook & Verification Manual
**Platform:** `telemetry-hub` (Ecological Telemetry Hub & Cognitive Digital Twin)  
**Standard:** Industrial Space-Grade Systems Engineering  
**Version:** 1.0.0 (Post Mega-Schema v2.0 Convergence)  

---

## 📑 Tabla de Contenidos
1. [Arquitectura del Sistema](#1-arquitectura-del-sistema)
2. [Requisitos Previos y Toolchain](#2-requisitos-previos-y-toolchain)
3. [Configuración de Entorno (.env)](#3-configuración-de-entorno-env)
4. [Despliegue de Infraestructura Local (Docker)](#4-despliegue-de-infraestructura-local-docker)
5. [Gestión y Verificación de Base de Datos (TimescaleDB)](#5-gestión-y-verificación-de-base-de-datos-timescaledb)
6. [Iniciación de Servicios](#6-iniciación-de-servicios)
   - [6.1 Servidor Backend (Axum / Tokio)](#61-servidor-backend-axum--tokio)
   - [6.2 Frontend Industrial (Leptos / WebAssembly)](#62-frontend-industrial-leptos--webassembly)
7. [Inyección de Telemetría End-to-End (Dummy Publisher)](#7-inyección-de-telemetría-end-to-end-dummy-publisher)
8. [Auditorías de Calidad, Linters y Pruebas Unitarias](#8-auditorías-de-calidad-linters-y-pruebas-unitarias)
9. [Verificación de APIs y Operabilidad Operativa](#9-verificación-de-apis-y-operabilidad-operativa)
10. [Simulación de Resiliencia y Poison Pills (DLQ)](#10-simulación-de-resiliencia-y-poison-pills-dlq)
11. [Guía de Solución de Problemas (Troubleshooting)](#11-guía-de-solución-de-problemas-troubleshooting)

---

## 1. Arquitectura del Sistema

La plataforma desacopla la ingestión física mediante un suscriptor reactivo a Google Cloud Pub/Sub, garantizando persistencia idempotente en TimescaleDB y transmisión en tiempo real hacia una interfaz HMI en WebAssembly:

```
[ Nodo Sensor ] (BME688, SCD41, BMV080)
      │ ESP-NOW (Protobuf 98 bytes)
      ▼
[ Gateway ESP32-P4 / C6 ] (Inyección de metadatos de red)
      │ Wi-Fi / Ethernet (HTTPS / mTLS)
      ▼
[ GCP Pub/Sub Topic: room-telemetry ]
      │ Pull Subscriber Asíncrono (tokio worker)
      ▼
┌─────────────────────────────────────────────────────────────┐
│                 BACKEND RUST (Axum + Tokio)                 │
│                                                             │
│  1. Decodificación Protobuf (prost)                         │
│  2. Validación Estricta UUID (Poison Pill Guard)            │
│  3. Time-Sync Fallback (measured_at_ms == 0 -> Utc::now())  │
│  4. Enrutamiento Errores -> telemetry_dlq                   │
│  5. Ingestión Idempotente -> TimescaleDB (33 campos)        │
│  6. Actualización Digital Twin en RAM (LruCache 10K)         │
│  7. Broadcast WebSockets (/api/ws) protegido con JWT        │
└─────────────────────────────────────────────────────────────┘
      │                                       │
      ▼                                       ▼
[ TimescaleDB Hypertable ]          [ HMI Leptos (Wasm) ]
(telemetry + telemetry_dlq)         (Gruvbox Hard Dark)
                                    - Badge "Calibrando"
                                    - Alerta Óptica BMV080
                                    - Latencia de Red
                                    - Atenuación de Stale Data
```

> [!IMPORTANT]
> **Nota de Arquitectura de Seguridad (Terminación TLS Externa):**  
> El binario Axum del backend opera en HTTP/1.1 plano por diseño cloud-native para optimizar el rendimiento y delegar la criptografía de capa de transporte a la infraestructura. En despliegues de producción, es **estrictamente obligatorio** utilizar un terminador TLS externo (como Google Cloud Run, Envoy, AWS ALB o Nginx Reverse Proxy) que gestione los certificados SSL/TLS y aplique cifrado HTTPS/WSS de extremo a extremo hacia los clientes.

---

## 2. Requisitos Previos y Toolchain

Asegúrese de contar con las siguientes herramientas instaladas y verificadas:

```bash
# 1. Toolchain Rust (estable con componentes obligatorios)
rustc --version
cargo --version
rustup component add rustfmt clippy
rustup target add wasm32-unknown-unknown

# 2. Compilador y Bundler Wasm para Frontend Leptos
cargo install trunk

# 3. Herramientas de Auditoría y Seguridad
cargo install cargo-audit
cargo install cargo-deny

# 4. Motor de Contenedores y Emuladores
docker --version
docker compose version
```

---

## 3. Configuración de Entorno (.env)

Cree un archivo `.env` en la raíz del repositorio (`telemetry-hub/.env`) basándose en [.env.example](file:///run/media/ecotech/1c5134f3-88ae-49dd-8284-12d45f74828b/home/oscar/Workspaces/telemetry-hub/.env.example):

```bash
cp .env.example .env
```

### Variables de Configuración

| Variable | Valor por Defecto (Desarrollo) | Descripción |
|---|---|---|
| `DATABASE_URL` | `postgres://postgres:password@localhost:5432/telemetry_hub` | Cadena de conexión al cluster TimescaleDB. |
| `PUBSUB_EMULATOR_HOST` | `localhost:8085` | Host del emulador local de GCP Pub/Sub. |
| `PUBSUB_SUBSCRIPTION_NAME` | `projects/oscar-bio-dev-project/subscriptions/room-telemetry-sub` | Nombre canónico de la suscripción Pub/Sub. |
| `GEMINI_API_KEY` | *(Opcional en dev / Requerido para Chat IA)* | Llave de API para el modelo generativo del Gemelo Digital. |
| `JWT_SECRET` | `<YOUR_32_BYTE_JWT_SECRET>` | Clave criptográfica para firma HMAC-SHA256 (Mínimo 32 bytes). |
| `ADMIN_API_KEY` | `<YOUR_OPERATOR_API_KEY>` | Clave secreta para login de operador (`/api/auth/login`). |
| `PORT` | `3000` | Puerto de escucha HTTP/WebSocket del backend. |
| `APP_HOST` | `0.0.0.0` | Dirección de enlace del servidor Axum. |
| `CORS_ORIGIN` | `http://localhost:8080` | Origen permitido para solicitudes web del HMI Leptos. |
| `RUST_LOG` | `backend=debug,shared=debug,tower_http=info` | Nivel de filtro de observabilidad estructurada (`tracing`). |

---

## 4. Despliegue de Infraestructura Local (Docker)

Inicie los contenedores de TimescaleDB y del Emulador de Google Cloud Pub/Sub:

```bash
# Iniciar servicios en segundo plano
docker compose up -d

# Verificar estado de los contenedores
docker compose ps
```

Debe observar dos contenedores en estado `Up`:
- `telemetry_hub_db`: Puerto `127.0.0.1:5432`
- `pubsub_emulator`: Puerto `0.0.0.0:8085`

---

## 5. Gestión y Verificación de Base de Datos (TimescaleDB)

Las migraciones se ejecutan de manera automática al inicializar el backend mediante `sqlx::migrate!("./migrations")`.

Para auditar o verificar el estado de las tablas directamente en PostgreSQL:

```bash
# Conexión interactiva a TimescaleDB
docker exec -it telemetry_hub_db psql -U postgres -d telemetry_hub
```

### Consultas de Verificación Operativa

```sql
-- 1. Verificar hipertables activas de TimescaleDB
SELECT hypertable_name, num_chunks FROM timescaledb_information.hypertables;

-- 2. Inspeccionar columnas del Mega-Esquema v2 (33 campos)
\d telemetry

-- 3. Consultar las últimas lecturas persistidas
SELECT 
    event_id, 
    device_id, 
    measured_at, 
    temperature, 
    humidity, 
    iaq, 
    eco2,
    is_calibrating,
    is_laser_obstructed,
    status 
FROM telemetry 
ORDER BY measured_at DESC 
LIMIT 5;

-- 4. Inspeccionar la cola de paquetes corruptos (DLQ)
SELECT id, ingested_at, error_reason, gateway_id FROM telemetry_dlq ORDER BY ingested_at DESC LIMIT 5;
```

---

## 6. Iniciación de Servicios

### 6.1 Servidor Backend (Axum / Tokio)

En la raíz del proyecto:

```bash
cargo run -p backend
```

**Comportamiento esperado en logs:**
1. Inicialización del suscriptor de observabilidad `tracing`.
2. Conexión al pool de conexiones de base de datos (`TimescaleDB`).
3. Ejecución y validación de migraciones SQL (incluyendo `0006_mega_schema_v2_fields.sql`).
4. Inicialización del estado concurrente (`Digital Twin` con capacidad de 10,000 nodos).
5. Auto-creación del Topic y Suscripción en el Emulador de Pub/Sub (`pubsub_ready: true`).
6. Enlace del servidor Axum en `0.0.0.0:3000`.

### 6.2 Frontend Industrial (Leptos / WebAssembly)

En una terminal separada:

```bash
cd frontend

# Servidor de desarrollo con recarga en caliente (Hot-Reload)
trunk serve --port 8080 --open
```

Para generar la compilación optimizada de producción (artefactos estáticos):

```bash
cd frontend
trunk build --release
```

---

## 7. Inyección de Telemetría End-to-End (Dummy Publisher)

Para validar el flujo completo sin requerir microcontroladores físicos conectados:

```bash
# En una terminal separada, ejecutar el inyector sintético
cargo run -p backend --bin dummy_publisher
```

**Lo que hace el inyector:**
- Inyecta cargas binarias Protobuf conformes al Mega-Esquema cada 2 segundos.
- Genera identificadores UUID canónicos válidos (`00000000-0000-0000-0000-00000000000X`).
- Simula datos fluctuantes de temperatura, humedad, calidad de aire (IAQ/eCO2), partículas (BMV080) y diagnósticos.
- Valida que el pipeline Pub/Sub -> Worker -> TimescaleDB -> WebSockets -> HMI funcione de extremo a extremo.

---

## 8. Auditorías de Calidad, Linters y Pruebas Unitarias

El proyecto impone tolerancia cero a advertencias (`-D warnings`). Ejecute este conjunto de validaciones antes de cualquier commit:

```bash
# 1. Comprobación estricta de formato
cargo fmt --all -- --check

# 2. Análisis estático pedante (Clippy)
cargo clippy --workspace --all-targets --all-features -- -D warnings -A clippy::cargo_common_metadata

# 3. Pruebas unitarias completas del workspace
cargo test --workspace

# 4. Verificación de compilación del target WebAssembly
cargo build -p frontend --target wasm32-unknown-unknown

# 5. Auditoría de vulnerabilidades de dependencias
cargo audit

# 6. Verificación de licencias
cargo deny check

# 7. Ejecución del hook de pre-commit de Propiedad Intelectual
.git/hooks/pre-commit
```

---

## 9. Verificación de APIs y Operabilidad Operativa

### 9.1 Health Check y Readiness Probes (Público)

Verifique el estado del runtime, el pool de TimescaleDB y la conectividad con Pub/Sub:

```bash
# Liveness Probe (Estado general, DB y Pub/Sub)
curl -s http://127.0.0.1:3000/health | jq

# Readiness Probe (Kubelet / Cloud Run startup check)
curl -i http://127.0.0.1:3000/ready
```

*Respuesta esperada en `/health`:*
```json
{
  "status": "healthy",
  "database": true,
  "pubsub": true
}
```

*Respuesta esperada en `/ready`:*
`HTTP/1.1 200 OK` con contenido `ready\n` (o `503 Service Unavailable` con `not ready\n` en caso de pérdida de conectividad a TimescaleDB).

### 9.2 Swagger / OpenAPI
Abra en el navegador:
```
http://127.0.0.1:3000/swagger-ui/
```

### 9.3 Autenticación Zero-Trust (Operator Login)
Obtenga un token JWT de operador mediante `POST /api/auth/login` con la cabecera `x-api-key`:
```bash
TOKEN=$(curl -s -X POST http://127.0.0.1:3000/api/auth/login -H "x-api-key: change_me_in_prod_operator_key" | jq -r .token)
echo "JWT Token: $TOKEN"
```

### 9.4 Consulta del Gemelo Digital (Público con Rate-Limiting)
```bash
curl -s http://127.0.0.1:3000/api/digital-twin | jq
```

### 9.5 Inspección de Alertas Críticas (DLQ - Protegido)
La cola DLQ requiere autenticación de operador:
```bash
curl -s -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3000/api/dlq/recent | jq
```

### 9.6 Conexión de Streaming WebSocket (Público Read-Only)
Conéctese utilizando `wscat` o inspeccione desde el frontend Leptos sin credenciales en la URL:
```bash
wscat -c "ws://127.0.0.1:3000/api/ws"
```

---

## 10. Simulación de Resiliencia y Poison Pills (DLQ)

La arquitectura garantiza que paquetes maliciosos o corruptos **nunca causen pánico (`panic!`) ni bloqueen el pipeline asíncrono**:

### Prueba de Inyección de Poison Pill
Para simular un paquete corrupto, envíe un payload con un `event_id` no conforme al estándar UUID al emulador:

```bash
# Publicación directa de datos inválidos en el topic de Pub/Sub
curl -X POST http://localhost:8085/v1/projects/oscar-bio-dev-project/topics/room-telemetry:publish \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      {
        "data": "aW52YWxpZF9wYXlsb2FkX25vdF9wcm90b2J1Zg=="
      }
    ]
  }'
```

**Resultado esperado:**
1. El backend detecta `Poison Pill (Protobuf inválido)` o `Poison Pill (event_id no es un UUID válido)`.
2. El mensaje es confirmado (`ACK`) ante Pub/Sub para no congestionar la cola.
3. El paquete binario crudo se persiste en la tabla `telemetry_dlq` de PostgreSQL con la razón del descarte.
4. El panel "Terminal de Alertas Críticas" en el frontend Leptos muestra la nueva fila en tiempo real tras el siguiente intervalo de sondeo (5s).

---

## 11. Guía de Solución de Problemas (Troubleshooting)

### A. Fallo de Conexión a Base de Datos
- **Síntoma:** Error `DATABASE_URL must be set` o `Connection refused (telemetry_hub_db:5432)`.
- **Acción:**
  ```bash
  docker compose ps
  docker compose restart db
  ```
  Verifique que el puerto `5432` no esté ocupado por una instancia local de PostgreSQL en el host (`sudo ss -tulpn | grep 5432`).

### B. Fallo de Conexión a Pub/Sub
- **Síntoma:** El worker se queda en bucle de reintento con backoff exponencial: `Fallo al inicializar Pub/Sub`.
- **Acción:**
  Asegúrese de que el emulador esté respondiendo en el puerto 8085:
  ```bash
  curl http://localhost:8085/
  ```
  Si no responde, reinicie el contenedor: `docker compose restart pubsub-emulator`.

### C. Error en Validación de Cabecera Pre-Commit
- **Síntoma:** El commit falla con `❌ Falta la cabecera privativa en: ...`.
- **Acción:**
  Asegúrese de que cada archivo `.rs` comience con:
  ```rust
  // Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.
  ```

### D. Datos "Stale" (Atenuados) en el HMI
- **Síntoma:** Las tarjetas de telemetría se muestran atenuadas (grayscale/opacidad 0.5) y con el aviso `Stale: updated X.Xs ago`.
- **Causa:** El nodo o el `dummy_publisher` dejó de transmitir por más de 5 segundos.
- **Acción:** Inicie o verifique que el `dummy_publisher` esté activo y publicando.
