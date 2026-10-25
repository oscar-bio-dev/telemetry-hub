# Auditoría GUI — telemetry-hub (Leptos 0.6 CSR / Wasm)

> Alcance: análisis y arquitectura. **No se modificó ningún `.rs` ni `.css`.**
> Commit auditado: `da84b41`. Líneas referidas a ese estado del árbol.

## 0. Nota de línea base (el código ya supera el enunciado)

Varias premisas del encargo ya no coinciden con el repositorio. Esta auditoría se basa en el código real:

| Premisa del encargo | Estado real |
|---|---|
| `.container { max-width: 800px }` | `style.css:44-51` → `1320px` |
| Sin barra de navegación global | Existe `components/nav.rs:1-49` (`SectionNav`), montada en `app.rs:30` |
| Solo hay ruta de telemetría | `app.rs:33-37`: `/`, `/hardware`, `/hpc`, `/field-research`, `/telemetry/room` + 404 |
| Home sin Hardware/HPC/Biología | `home.rs:201-247` enlaza los 3 módulos; existen `hardware.rs` (430 L), `hpc.rs` (394 L), `field_research.rs` (193 L) |
| Sin sparklines | `room_monitoring.rs:38-117` (`SparklineCanvas`, SVG) — solo temperatura y CO₂ |
| Sin `tabular-nums`, `:focus-visible`, `prefers-reduced-motion` | `style.css:233-240`, `:766-769`, `:772-790` ya existen |
| `scope="col"` ausente | Presente en `room_monitoring.rs:671-674` y `spec_sheet.rs:50-54` |
| Emojis como iconografía | Home usa SVG (`home.rs:11-65`); quedan emojis residuales (`room_monitoring.rs:434,445`) |
| Sin MathML | `hpc.rs:330-339` (`<math class="native-mathml">`) + `style.css:1433-1467` |
| Estilos/JS inline masivos | Solo 3 `style=` (`room_monitoring.rs:110-112`); **0** `onmouseover/onmouseout` |

Los huecos reales están en: cobertura del schema, rendimiento reactivo, resiliencia (WS/DLQ/chat), a11y fina y organización CSS.

---

## 1. Resumen ejecutivo y matriz de madurez

| # | Dimensión | Nota | Justificación técnica |
|---|---|---|---|
| 1 | Datos omitidos (Domain vs UI) | **4/10** | Se renderizan 18 de 33 campos; faltan `status`, `system_error_bitmask`, `is_pm_out_of_range`, `laser_runtime`, `bvoc`, `tvoc`, `pm1_0`, 3 conteos, `node_sequence`, `gateway_id`. `GatewayHealthEvent` no se consume ni se emite por WS (`backend/src/api/ws.rs:35-52` solo difunde `TelemetryPayload`). Sparklines solo en 2 de 13 métricas, sin deltas ni umbrales. |
| 2 | Arquitectura de información / UX | **7/10** | Nav global, 5 rutas, home ejecutivo, SpecSheet y diagramas SVG. Faltan BOM/esquemáticos descargables, benchmarks AVX-512, vista de flota/detalle por nodo, y la ruta de telemetría mezcla flota + DLQ sin jerarquía. |
| 3 | Reactividad Leptos / rendimiento Wasm | **4/10** | El bloque de flota (`room_monitoring.rs:399-613`) es un único closure que depende de `twin_state` **y** `current_time`: **reconstruye todo el DOM cada segundo** y en cada paquete. Clones del `HashMap` completo. Orden de tarjetas no determinista. Fuga de tareas WS al desmontar. |
| 4 | CSS / tipografía / a11y | **6/10** | Gruvbox con tokens semánticos parciales, tabular-nums, focus-visible y reduced-motion presentes. Débil: archivo monolítico de 1759 L, CSS muerto, tokens de estado por métrica en vez de por semántica, contraste `--accent-red` sobre `--bg-card` 3.82:1, sin `aria-live`, Google Fonts sin SRI/preconnect ni *self-hosting*. |
| 5 | Seguridad y resiliencia operativa | **3/10** | **Bug P0**: `room_monitoring.rs:239` envía la cabecera `Authorization` con un literal fijo en vez de el esquema `Bearer` seguido del JWT (`token`); la DLQ nunca se carga y el token se descarta con 401. Sin manejo de 429 en el chat, `Connected` se marca antes del `open`, reconexión sin backoff, JSON inválido descartado en silencio, JWT sin expiración visible. |

**Media global: 4.8/10.** La base visual es sólida; el mayor retorno está en (a) corregir el bug de DLQ, (b) granularizar la reactividad, (c) cubrir los 15 campos ausentes.

---

## 2. Hallazgos técnicos archivo por archivo

### 2.1 `frontend/Cargo.toml` / `Trunk.toml`
- `Cargo.toml:14-15` `reqwest` (con TLS/hyper para wasm) + `gloo-net` conviven: dos clientes HTTP/WS → binario Wasm mayor. Unificar con `gloo-net::http` o `web_sys::fetch`.
- `Cargo.toml:19` `tokio` (`sync`,`macros`) sin uso aparente en `src/` → peso muerto. Verificar con `cargo machete` antes de quitar.
- `Cargo.toml` no declara `web-sys` explícitamente aunque se usa (`web_sys::window`, `room_monitoring.rs:181`) y llega transitivamente; fijar features (`Window`, `Location`, `Storage`).
- `Trunk.toml:1-14` sin `[tools]`/`wasm-opt` ni perfil `release` con `opt-level="z"`, `lto`, `codegen-units=1`. Sin `data-trunk rel="rust" data-wasm-opt="z"` en `index.html`.
- `Trunk.toml:9-13` proxy WS/HTTP correcto; pero las URLs del front se *hardcodean* a `127.0.0.1:3000` cuando el host contiene `localhost` (`room_monitoring.rs:185-189, 231-235, 276-280`; `chatbot.rs:43-48`) → lógica duplicada 4 veces; ignora el proxy de Trunk.

### 2.2 `frontend/index.html`
- `:13` Google Fonts: sin `rel="preconnect"`, sin SRI (CSS dinámico de Google no admite SRI estable) ⇒ **recomendado self-host** WOFF2 (Fira Code, Inter) en `public/assets/fonts` con `@font-face { font-display: swap }`. Beneficios: privacidad, CSP `font-src 'self'`, operación offline en campo.
- `:5-8` metadatos OG sin `og:image`, `og:type`, `twitter:card`, `theme-color`, `color-scheme`.
- `:2` `lang="en"` pero la UI está en español → `lang="es"` (lectores de pantalla pronuncian mal). Mantener `lang="en"` solo en fragmentos en inglés.
- Sin `<noscript>` ni estado de carga previo al montaje de Wasm (pantalla en blanco durante la descarga).
- Sin `<meta http-equiv>`/CSP: delegar a cabeceras del backend, pero documentar `style-src` (los `style=` inline de `room_monitoring.rs:110-112` exigen `'unsafe-inline'` o *nonces*; eliminarlos permite CSP estricta).

### 2.3 `frontend/src/main.rs`
- `:6` `#![allow(clippy::wildcard_imports, too_many_lines, semicolon_if_nothing_returned)]` global: aceptable, pero `too_many_lines` oculta que `RoomMonitoring` tiene ~550 líneas. Dividir en componentes (§4) y limitar el `allow` a módulos.
- `:16-17` sin *error boundary*: un `expect` en `web_sys::window()` (`room_monitoring.rs:181,226,270`; `chatbot.rs:39`) produce *panic* → pantalla congelada. Usar helper `api_base()` con `Result`.

### 2.4 `frontend/src/app.rs`
- `:18-67` Layout correcto. `:19` `.container` envuelve cabecera, nav, main y footer; sin *skip-link* ("Saltar al contenido") ni `id` en `<main>` (`:32`).
- `:20-30` el "prompt" decorativo es `<div>`; el único `<h1>` es un enlace (`:27`) → en cada ruta el `<h1>` es "Oscar Mora" y las páginas usan `<h2>`; falta título de página dinámico (`leptos_meta::Title`, ya dependencia en `Cargo.toml:16` pero **no usada**).
- `:34-48` 404 correcto; añadir `Title`, y código HTTP no aplica en CSR.
- `:54-59` Barra de estado inferior **estática**: `"● STATUS: ONLINE"` (`:54`) es un literal, no refleja el estado real del WS/backend → falsa señal operativa (anti-patrón SCADA). Debe leer un contexto global `ConnectionStatus`.
- `:62` `<Chatbot/>` fuera de `<main>` pero dentro de `<Router>`: correcto; el estado del WS vive dentro de `RoomMonitoring`, por lo que **al salir de la ruta se pierde el historial** y no hay indicador de conexión en el `nav.rs`.

### 2.5 `frontend/src/components/nav.rs`
- `:11-46` `<nav aria-label>` + `<ul>` correcto. `:41` indicador `LIVE` es **siempre visible**, independiente del estado real (`.live-dot` anima infinito, `style.css:885`). Debe enlazarse a `ConnectionStatus` vía contexto (`provide_context`).
- `A` de `leptos_router` aplica `aria-current="page"` automáticamente; estilizar `[aria-current="page"]` en CSS (verificar presencia).

### 2.6 `frontend/src/pages/home.rs`
- `:11-65` iconos SVG inline repetidos cuatro veces con atributos idénticos (`viewBox`, `stroke`…) → extraer `<TechIcon kind=…/>`.
- `:156-199` perfil: enlaces externos `rel="noopener noreferrer"` ✔.
- `:201-247` tarjetas de módulos sin métricas "vivas" (p. ej. nº de nodos online, última lectura) → home no demuestra el sistema en operación.
- `:250-…` SpecSheets (`components/spec_sheet.rs`) con datos estáticos; no hay **BOM**, esquemáticos, STL de carcasa, ni enlaces a *datasheets*/repos de firmware por instrumento (WQM-V100, ESP32-P4/C6, MSPM0, RP2350).
- Sin sección de publicaciones/resultados de campo ni métricas (nº de especímenes, transectos, parcelas).

### 2.7 `frontend/src/pages/room_monitoring.rs` (archivo crítico)

| Línea | Hallazgo | Impacto |
|---|---|---|
| `:153-157` | 5 señales en un solo componente; estado de flota (`HashMap<String, DeviceState>`), red, auth y DLQ mezclados | Acoplamiento; imposible reutilizar/testear |
| `:160-172` | `create_effect` sin dependencias reactivas para registrar un `setInterval`; se ejecuta una vez pero semánticamente es *mount*. Debe ser `on_mount`/`create_effect` + `on_cleanup` fuera del efecto | Riesgo de duplicar timers si el efecto llegara a re-ejecutarse |
| `:163` | `current_time` global cada 1 s | Dispara cualquier closure que lo lea (ver `:400`) |
| `:177-215` | `login_operator`: JWT solo en memoria (bien), pero sin expiración, sin logout, sin indicador "sesión activa" | Operador no sabe cuándo se cierra la sesión; `:251` limpia el token silenciosamente |
| `:199` | `Ok(res)` sin `token` en el JSON → no hay mensaje de error (cae en silencio) | UI "no pasa nada" |
| `:218-263` | Polling DLQ cada 5 s con `reqwest::Client::new()` nuevo cada vez; sin cancelar peticiones en vuelo ni *backoff*; errores de red ignorados (`if let Ok`) | Fuga de conexiones, sin feedback de fallo |
| **`:239`** | `.header("Authorization", format!(...))` usa una cadena literal sin interpolar `token` | **Bug P0**: todas las llamadas devuelven 401 → `:248-252` borra el token → la consola DLQ se "re-bloquea" tras ≤5 s. Esperado: el esquema `Bearer` seguido del JWT (`token`) |
| `:267-354` | Efecto WS: lee `retry_trigger.track()` | OK para reintento, pero ver abajo |
| `:284-285` | `set_conn_status(Connected)` inmediatamente tras `WebSocket::open` (que solo crea el objeto, no espera `onopen`) | Estado "CONNECTED" falso durante el handshake |
| `:291` | `serde_json::from_str` con `if let Ok`: payload inválido se descarta sin contador ni log | Pérdida invisible de telemetría; añadir contador `parse_errors` |
| `:292-332` | `set_twin_state.update` clona `temp_history` y `co2_history` completos (`:303-306`) y `payload.clone()` (`:326`) por paquete | O(60) copias + alloc por mensaje; usar `get_mut` + `push_back` in-place |
| `:289-344` | `spawn_local` nunca se cancela al desmontar; al navegar fuera, el bucle sigue leyendo y reintentando cada 3 s (`:338-343`) contra señales ya liberadas | Fuga de conexión WS y CPU; acumula sockets si se entra/sale repetidamente |
| `:337-343` | Reintento fijo 3 s, sin *exponential backoff* ni *jitter* | Tormenta de reconexión si el backend cae |
| `:388-397` | `Show when=!twin_state.get().is_empty()` clona **todo el HashMap** (incl. históricos) solo para comprobar vacío | Usar `twin_state.with(\|m\| !m.is_empty())` o `create_memo` |
| **`:399-400`** | `twin_state.get().values().cloned()` + `current_time.get()` dentro del mismo closure | **Cada segundo y cada paquete se destruye y recrea el DOM de todas las tarjetas** (≈13 `MetricReadout` + 2 SVG por nodo): pierde *hover*, foco, selección de texto y reinicia animaciones CSS |
| `:399` | `HashMap::values()` → orden no determinista entre reconstrucciones | Las tarjetas **cambian de posición** aleatoriamente |
| `:407-415` | Latencia `ingested_at - measured_at` es latencia nodo→gateway, no de red hasta el navegador; etiqueta "Latencia Red" (`:599`) es engañosa. Con relojes desincronizados puede ser negativa (`:409` la oculta como "<1 ms") | Métrica metrológicamente incorrecta |
| `:401-402` | Umbral *stale* fijo 5 s, global; no por cadencia del nodo (deep-sleep → minutos) | Falsos "stale" en nodos de bajo consumo |
| `:417-418` | Solo `is_calibrating` e `is_laser_obstructed`; ignora `status`, `system_error_bitmask`, `is_pm_out_of_range` | Un `HardwareError` pasa **invisible** |
| `:431-439, 442-451` | Ramas `if/else` con `view!{<span/>}.into_any()` para "nada" → nodos vacíos; usar `<Show>` o `then(\|\| …)` | Ruido DOM |
| `:434, 445` | Emojis `⏳` `⚠️` como icono de estado; renderizado dependiente de la fuente del SO, sin `aria-label`, color no controlable por CSS | Inconsistencia visual / lectura de pantalla ("warning sign") |
| `:454-594` | 13 bloques `MetricReadout` repetitivos con tolerancias *hardcodeadas* como `&'static str` | Sin fuente única de metadatos; tolerancias no verificables vs. datasheet; ver §3/§4 (`SensorSpec`) |
| `:461-466, 477-482` | Sparkline solo temp/CO₂; `data=temp_history.clone()` otra copia | Sin tendencia para humedad, PM, IAQ, pH, OD, batería |
| `:529` | `eco2` usa `{:.0}` sin newtype | OK, pero sin validación de rango |
| `:456-593` | Sin umbrales/semáforos (OK/WARN/FAULT) por métrica; el color es de "tema", no de estado (`--status-*` existe pero no se aplica a valores) | El operador no distingue valor normal de crítico |
| `:603-607` | "Live Data (0s)" literal | No muestra edad real mientras está *live* |
| `:597-609` | Sin `node_sequence`/gap detection, sin `gateway_id`, sin `event_id` | No se detecta pérdida de paquetes |
| `:617-697` | DLQ: bloqueada/desbloqueada con `Show`; correcto zero-trust, pero: sin botón "Bloquear", sin timeout de sesión, sin contador de registros, sin truncado/copia del Base64 (`:684-686`, celdas de ancho 220 px `style.css:507`), sin ordenación/filtro, `<td>` con claves ausentes en `For` | Usabilidad forense baja |
| `:657-659` | `login_error` sin `role="alert"` | Lectores de pantalla no anuncian el error |
| `:633-644` | `<input type=password>` sin `<label>` ni `autocomplete="off"`/`aria-describedby` | Falla WCAG 1.3.1 / 3.3.2 |
| `:678-689` | `dlq_records.get().into_iter().map().collect_view()` — sin `<For>` ni clave (`id`) | Re-render completo de la tabla cada 5 s aunque no haya cambios |
| `:11-20` | `#[allow(dead_code)]` en `DlqRecordDto`; `event_id` nunca mostrado | Campo útil para trazabilidad |
| `:665` | Mensaje `[OK] No hay Poison Pills` también aparece si el **fetch falla** (porque `dlq_records` queda vacío) | **Falso negativo operativo**: un fallo de red/401 se muestra como "todo bien" |

### 2.8 `frontend/src/components/chatbot.rs`
- `:29-97` `create_action` con `async`: ✔ manejo de `Ok/Err` y `status`.
- `:71-82` Rama no-2xx: pinta `API Error {status}: {err_text}` con el cuerpo crudo. **No hay tratamiento específico de 429** (`backend/src/main.rs:154-160`: ~5 req/min, ráfaga 2) ni lectura de `Retry-After`; el usuario recibe texto técnico y puede reintentar inmediatamente agravando el bloqueo. Propuesta: estado `cooldown_until`, botón *deshabilitado* con cuenta atrás.
- `:93-95` `set_input("")` solo tras la respuesta → el texto del usuario permanece en el campo mientras carga y **se puede reenviar** (doble envío; no se comprueba `is_loading` en `:141-144, 149-153`).
- `:107` historial `Vec<ChatMessage>` sin límite ni persistencia; `<For>` ausente (re-render total del historial por cada mensaje, `:107-125`); sender como `String` ("User"/"AI"/"System") comparado con cadenas (`:108-109`) → usar `enum Sender`.
- `:106-126` sin `role="log"`/`aria-live="polite"` en el contenedor de mensajes, sin auto-scroll al último mensaje.
- `:101-104` cabecera sin botón cerrar ni `role="dialog"`/`aria-modal`; la ventana usa `display:none` (`style.css:589`) → ✔ no focusable cuando cerrada, pero sin gestión de foco al abrir/cerrar ni `Escape`.
- `:160-165` FAB con `aria-label` ✔, falta `aria-expanded`/`aria-controls`.
- `:134-145` `<input>` sin `<label>` (solo `placeholder`), sin `maxlength` (el backend debería limitar; reflejarlo en UI).
- `:36-48` URL base duplicada (ver 2.1).
- No hay *streaming* de la respuesta ni render Markdown: respuestas largas del LLM se muestran como texto plano.
- No se envía contexto de telemetría/ruta actual al asistente (oportunidad: "explica este nodo").

### 2.9 `frontend/public/assets/style.css` (1759 L)
- **Organización**: monolítica, secuencia por "Etapas" (comentarios `Etapa 2…`), no por capas. Recomendado `@layer reset, tokens, base, layout, components, utilities, pages;` y división en ficheros (`tokens.css`, `base.css`, `layout.css`, `components/*.css`, `pages/*.css`) que Trunk puede concatenar (múltiples `<link data-trunk rel="css">`).
- **Tokens** (`:1-25`): hay paleta primitiva (`--bg-*`, `--accent-*`) y 6 tokens semánticos (`--status-ok/warn/fault`, `--data-primary`, `--bio-accent`, `--surface-instrument`), pero **no se usan consistentemente**: p. ej. `.metric-temp .metric-readout-value { color: var(--accent-yellow) }` (`:1058`) y `.chatbot-*` usan primitivas directamente; 30 ocurrencias de `rgba(...)` *hardcodeadas* en lugar de `color-mix()`/tokens `--*-alpha`.
- **Faltan tokens**: espaciado (`--space-*`), radio, elevación, duración (`--dur-fast`), tipografía (`--text-xs…`), z-index, estado de nodo (`--state-stale`, `--state-calibrating`, `--state-fault`, `--state-offline`), series (`--series-1..8`), y *surface* (`--surface-0/1/2`).
- **CSS muerto**: `.console-shell` (`:54-59`, `:61-66`), `.telemetry-grid` (`:~300`), `.metric-row`, `.metric-value` (`:233-240`) no se usan en ningún `.rs` (grep: 0 resultados). Eliminar.
- **Tipografía**: `.tabular-nums` (`:233-240`) existe y se aplica en readouts, tablas y rango sparkline ✔; pero el selector solo se aplica si cada componente añade la clase → centralizar en `.metric-readout-value, td, time, output { font-variant-numeric: tabular-nums lining-nums }`. Fira Code ya es monoespaciada, por lo que el salto de dígitos es menor; el riesgo real está en `Inter` (sans) para contadores (`.latency-badge`, edad). Añadir `font-feature-settings: "zero","ss01"` (cero con barra) para distinguir `0/O` en IDs. Escala tipográfica ad-hoc (`0.7rem`, `0.75rem`, `0.82rem`, `0.85rem`, `1.8rem`…) → definir escala modular.
- **MathML**: `.native-mathml` (`:1449-1470`) usa `Cambria Math, STIX Two Math, Latin Modern Math` — correcto; Firefox/Chrome ≥109 lo soportan. Falta `@font-face` de *Latin Modern Math* como respaldo cuando el SO no tiene fuente matemática.
- **Animaciones infinitas**: `spin` (`:330`), `pulse-opacity` (`:339`), `pulse-border` (`:392`), `blink-cursor` (`:96`), `live-pulse-dot` (`:885`). El bloque `prefers-reduced-motion` (`:772-790`) las neutraliza (`animation-iteration-count: 1 !important` + `animation: none`) ✔ — **cumple**. Matiz: `.live-dot` (`:885`) no está en la lista explícita (queda cubierto solo por el comodín `0.01ms`), y `.pulse-text` con `animation: none` deja la señal "ESTABLISHING LINK" sin indicación de actividad → añadir alternativa estática (puntos/texto).
- **`:focus-visible`** (`:766-769`) global ✔ (contraste `--data-primary` 6.09:1 sobre `--bg-hard`). Pero `.dlq-key-input:focus` (`:548`) y `.chatbot-input:focus` (`:720`) usan `:focus` (se activa con ratón) y pueden quitar `outline`; verificar que no hacen `outline: none` sin sustituto.
- **Contraste** (calculado WCAG 2.x):

  | Par | Ratio | Veredicto |
  |---|---|---|
  | `--fg-muted #a89984` / `--bg-hard` | 5.90 | AA ✔ |
  | `--fg-muted` / `--bg-card #32302f` | 4.72 | AA ✔ (justo) |
  | `--accent-red #fb4934` / `--bg-hard` | 4.77 | AA ✔ |
  | `--accent-red` / `--bg-card` | **3.82** | ✘ AA texto normal (≥4.5); solo válido para texto grande ≥ 18.66 px bold. Afecta `.metric-co2` (`:1058+`) y `.alert-laser-obstructed` |
  | `--border-color #504945` / `--bg-hard` | **1.86** | ✘ WCAG 1.4.11 (componentes UI ≥3:1) para bordes de `input`/botones |
  | `--accent-blue` / `--bg-card` | 4.88 | ✔ |
  | `--bg-hard` sobre `--status-warn` (badge/botones) | 9.67 | ✔ |

  Además, **el estado nunca se comunica solo por color**: `stale-data` (`:347`) usa `opacity:.5` + `grayscale` (reduce contraste efectivo de texto muted a <3:1) sin texto alternativo salvo la línea inferior.
- **`.stale-data`** (`:347-351`) con `opacity: 0.5` degrada el contraste del texto por debajo de AA; mejor atenuar solo el valor y añadir etiqueta `STALE`.
- **Sin `color-scheme: dark`** en `:root`: controles nativos (scrollbar, input) renderizan con esquema claro; añadir `color-scheme: dark`.
- **Responsivo**: breakpoints 600/820/860/900 px dispersos; sin `container queries` para las tarjetas de nodo (el ancho depende de la rejilla, no del viewport).
- **Chatbot**: `width: 360px` fijo (`:591`) → desborda en móviles <360 px; usar `min(360px, calc(100vw - 2rem))`.
- **Print**: sin `@media print` para el portafolio (CV técnico imprimible).
- **Imagen**: `.chatbot-fab` `url('/public/assets/img/ia-avatar.png')` (`:751`) sin dimensiones/optimización (PNG); considerar WebP/SVG.

### 2.10 `shared/src/lib.rs` (contraste dominio ↔ UI)
- `TelemetryPayload` (`:410-523`) define **33 campos**; la UI usa 18 (ver §3).
- `NodeStatus` (`:530-559`) con `as_str()` y `Display` listos para UI; **no se usa en el frontend** (`grep NodeStatus frontend/src` → 0).
- `GatewayHealthEvent` (`:769-803`): 10 campos (SD, heap, uptime, alert_message) → **0 usos en el frontend** y no hay canal WS/endpoint para ellos; la UI no puede mostrar salud del ESP32-P4.
- Newtypes (`Temperature`, `Co2`, … `:39-398`) exponen `MIN/MAX` (p. ej. `Ph::MIN/MAX`, `:46-47`) → **los rangos de operación pueden derivarse del dominio** en vez de *hardcodear* cadenas de tolerancia en la UI. `eco2`, `bvoc`, `tvoc`, conteos y `laser_runtime` son `f64` sin newtype (sin validación ni rango reutilizable).
- `DateTime<Utc>` serializado ISO-8601; el frontend lo parsea con `chrono` (compartido) ✔, pero `last_updated_ms` usa `Date::now()` (reloj del navegador) mezclado con `measured_at` (reloj del nodo).

---

## 3. Cobertura del Mega-Schema (33 campos de `TelemetryPayload`)

Leyenda: ✔ renderizado · ◐ parcial · ✘ no renderizado.

| # | Campo | Tipo | Render actual | Propuesta HMI |
|---|---|---|---|---|
| 1 | `protocol_version` | u32 | ✘ | Pie de nodo `proto v{p}.{s}`; alerta si ≠ esperado |
| 2 | `schema_version` | u32 | ✘ | Ídem (tooltip en cabecera de tarjeta) |
| 3 | `event_id` | String | ✘ | Copiable en panel "Detalle de paquete" / correlación con DLQ |
| 4 | `gateway_id` | String | ✘ | Chip de agrupación "GW …" + enlace a `GatewayHealthPanel` |
| 5 | `device_id` | String | ✔ `:428` | Mantener; clave de `<For>` |
| 6 | `node_sequence` | u32 | ✘ | Detector de huecos: Δseq>1 → badge `LOSS n`; sparkline de pérdidas |
| 7 | `measured_at` | DateTime | ◐ `:407` (solo latencia) | Reloj de nodo + *clock skew* vs navegador |
| 8 | `ingested_at` | DateTime | ◐ `:407` | Latencia nodo→gateway separada de gateway→UI |
| 9 | `temperature` | Temperature | ✔ `:454` + sparkline | Añadir Δ/min, umbral, banda ± |
| 10 | `humidity` | Humidity | ✔ `:486` sin sparkline | Sparkline + Δ |
| 11 | `pressure` | Pressure | ✔ `:536` sin sparkline | Sparkline + tendencia barométrica (3 h) |
| 12 | `gas_resistance` | GasResistance | ✔ `:546` | Sparkline log-escala |
| 13 | `iaq` | Iaq | ✔ `:516` | Gauge 0–500 con bandas BSEC (0-50 …>300) |
| 14 | `co2` | Co2 | ✔ `:470` + sparkline | Umbrales 800/1000/1500 ppm, Δ |
| 15 | `pm1_0` | Pm1_0 | ✘ | `PmSizeBars` (PM1/2.5/10 agrupado) |
| 16 | `pm2_5` | Pm2_5 | ✔ `:496` | Barra con límites OMS 2021 (15 µg/m³ 24 h) |
| 17 | `pm10_0` | Pm10_0 | ✔ `:506` | Barra con límite OMS (45 µg/m³ 24 h) |
| 18 | `ph` | Ph | ✔ `:556` | Escala 0–14 con gradiente + sparkline |
| 19 | `dissolved_oxygen` | DissolvedOxygen | ✔ `:566` | Sparkline + % saturación (con T y P) |
| 20 | `battery_mv` | u32 | ✔ `:576` (mV crudo) | Gauge de SoC LiFePO₄ (curva) + autonomía estimada |
| 21 | `sleep_cycles` | u32 | ✔ `:586` | Mantener; derivar duty-cycle |
| 22 | `is_calibrating` | bool | ✔ badge `:431` | Mantener con icono SVG |
| 23 | `eco2` | f64 | ✔ `:526` | Sparkline (virtual BSEC; marcar "estimado") |
| 24 | `bvoc` | f64 | ✘ | Readout ppm + sparkline |
| 25 | `tvoc` | f64 | ✘ | Readout ppb + sparkline |
| 26 | `pm1_0_count` | f64 | ✘ | Histograma de conteo #/m³ por tamaño (`PmSizeBars`, escala log) |
| 27 | `pm2_5_count` | f64 | ✘ | Ídem |
| 28 | `pm10_0_count` | f64 | ✘ | Ídem |
| 29 | `is_laser_obstructed` | bool | ✔ `:442` | Mantener; pasar a `DiagnosticsStrip` |
| 30 | `is_pm_out_of_range` | bool | ✘ | Badge `OVER-RANGE` sobre PM + valores marcados como saturados |
| 31 | `laser_runtime` | f64 (s) | ✘ | Horas de vida del láser vs. MTBF (barra de vida útil) |
| 32 | `system_error_bitmask` | u32 | ✘ | Decodificador de bits → lista de fallos (tabla bit→descripción) |
| 33 | `status` | NodeStatus | ✘ | **Estado primario del nodo** (`MONITORING/CALIBRATING/SELF_TESTING/HARDWARE_ERROR`) en cabecera |

**Cobertura: 18/33 usados (54.5 %); 15 sin visualizar. `GatewayHealthEvent`: 0/10 campos.**
Los *dots* de cabecera (`:423-425`) son decorativos (rojo/amarillo/verde fijos) y deberían ser el **LED de estado real** del nodo.

---

## 4. Especificación de nuevos componentes Leptos (SVG/Canvas 2D, sin dependencias JS)

Convenciones: props con `#[prop(into)]`, señales de solo lectura (`Signal<T>`/`ReadSignal<T>`), cero `style=` dinámico excepto custom properties (`style:--v=…`), todas las gráficas en SVG con `viewBox` + CSS; Canvas 2D (vía `web_sys::CanvasRenderingContext2d`) solo si > ~2 000 puntos.

### 4.1 Telemetría

**Modelo de estado propuesto (reemplaza `DeviceState`)**
```rust
pub struct NodeStore { pub nodes: RwSignal<Vec<NodeId>> /* orden estable */,
                       pub by_id: StoredValue<HashMap<NodeId, NodeSignals>> }
pub struct NodeSignals {                 // una señal por unidad de render
    pub latest:  RwSignal<TelemetryPayload>,
    pub last_rx: RwSignal<f64>,          // ms, Date::now()
    pub series:  HashMap<MetricKey, RwSignal<RingBuffer<f64, 120>>>,
    pub seq_gap: RwSignal<u32>,
}
```
Un `RingBuffer` de capacidad fija (`[f64; N]` + `head`) evita `VecDeque::clone`.

| Componente | Props | Señales / lógica | Implementación |
|---|---|---|---|
| `NodeStatusLed` | `status: Signal<Option<NodeStatus>>`, `bitmask: Signal<Option<u32>>`, `stale: Signal<bool>` | `Memo` → enum `UiState{Ok,Warn,Fault,Stale,Offline}` | `<svg>` 12×12 círculo; `class:state-fault=…`; `<title>` accesible + texto `visually-hidden` |
| `ErrorBitmaskDecoder` | `mask: Signal<u32>`, `table: &'static [(u32,&'static str)]` | `Memo` → `Vec<&str>` | Lista `<ul role="list">` con chips; `0` → "Sin errores" |
| `MetricReadout` (v2) | `spec: &'static SensorSpec`, `value: Signal<Option<f64>>`, `series: Signal<RingBuffer>`, `thresholds: Thresholds` | `Memo` de estado (`Ok/Warn/Fault`) y `Δ/min`; **solo el nodo de texto del valor** es reactivo (`{move || fmt(value.get())}`) | `SensorSpec { label, unit, decimals, tolerance, range:(f64,f64), source: &str }` derivado de los newtypes (`Ph::MIN/MAX`…) |
| `Sparkline` (v2) | `data: Signal<RingBuffer>`, `domain: Option<(f64,f64)>`, `bands: &'static [Band]`, `unit` | `Memo` que genera `points` (`String`) una vez por cambio; `last` marcador | `<polyline>` + `<path>` de área; bandas como `<rect>`; `vector-effect: non-scaling-stroke`; color vía clase (`.series-1`), **sin `style=`** |
| `TrendDelta` | `series`, `window_s: u32` | `Memo` regresión lineal simple → `▲/▼` SVG + `%/min` | `<svg>` triángulo + texto tabular |
| `PmSizeBars` | `pm1,pm2_5,pm10: Signal<Option<f64>>`, `counts: Signal<[Option<f64>;3]>`, `out_of_range: Signal<bool>` | `Memo` normaliza vs límites OMS | `<svg>` barras horizontales + marca de límite; segunda fila con conteo (escala log10); badge `OVER-RANGE` |
| `IaqGauge` | `iaq: Signal<Option<f64>>` | `Memo` → ángulo | Arco SVG 270° con 6 segmentos BSEC |
| `BatteryGauge` | `mv: Signal<Option<u32>>`, `chem: Chemistry` | SoC por curva de tabla | `<svg>` batería + `%` + estimación autonomía |
| `LaserLifeBar` | `runtime_s: Signal<Option<f64>>`, `rated_h: f64` | `Memo` % vida | Barra `<svg>`; alerta >80 % |
| `SequenceGapBadge` | `seq: Signal<u32>` | `create_memo(prev)` detecta Δ>1, acumula pérdidas | Texto + `aria-live="polite"` |
| `LinkQuality` | `measured_at`, `ingested_at`, `rx_at` | 3 latencias separadas (nodo→GW, GW→UI) + *skew* | Mini `<dl>` |
| `GatewayHealthPanel` | `gw: Signal<Option<GatewayHealthEvent>>` | `Memo` de % SD libre, heap, uptime humanizado | Tarjeta con barras SVG; `alert_message` en `role="alert"`; requiere backend: nuevo `type` en el WS (`{"kind":"gateway_health",…}`) o `GET /api/gateway/health` |
| `ConnectionBanner` | `status: Signal<ConnectionStatus>`, `retry_in: Signal<Option<u32>>`, `parse_errors: Signal<u32>` | Contexto global (`provide_context`) | Reemplaza literal `app.rs:54` y `LIVE` de `nav.rs:41` |
| `DlqConsole` | `token: RwSignal<Option<Token>>`, `records: Signal<Vec<DlqRecordDto>>`, `fetch_state: Signal<FetchState>` | `FetchState{Idle,Loading,Ok,Err(String)}` para distinguir "vacío" de "fallo"; temporizador de sesión | Tabla con `<For key=\|r\| r.id.clone()>`, botón "Copiar", "Bloquear" |

### 4.2 Hardware (`/hardware`)
| Componente | Props | Detalle |
|---|---|---|
| `InstrumentSheet` | `spec: &'static InstrumentSpec` (nombre, MCU, sensores, rango, incertidumbre, consumo, interfaz) | Extiende `SpecSheet`; añade `<dl>` y botones de descarga |
| `BomTable` | `items: &'static [BomItem]`, `currency` | `<table>` ordenable (señal de orden), total reactivo; `<caption>`, `scope="col"` |
| `SchematicViewer` | `src: &'static str` (SVG), `layers: &'static [Layer]` | SVG incrustado con `<use>`; zoom/pan con `pointer events` y `viewBox` reactivo; capas conmutables (checkbox) |
| `EnclosureViewer` | `views: &'static [(&str,&str)]` | Isométrica SVG pre-renderizada por vistas (sin WebGL); enlace STL |
| `PinoutDiagram` | `mcu: McuKind` | SVG de pines con `<title>` por pin |
| `PowerBudget` | `rails: &'static [Rail]`, `duty: RwSignal<f64>` | Slider → corriente media y autonomía (cálculo en Rust) |
| `BlockDiagram` (extraer) | `nodes, edges` | Los 3 SVG de `hardware.rs:18,89,159` están escritos a mano; generar desde datos |

### 4.3 HPC (`/hpc`)
| Componente | Props | Detalle |
|---|---|---|
| `ModelEquation` | `mathml: &'static str` o `View` | Envoltorio MathML + `aria-label` con alternativa textual (`hpc.rs:339`) |
| `BenchmarkChart` | `series: &'static [BenchSeries]` (scalar/AVX2/AVX-512, tamaños de malla), `log_y: bool` | Barras/líneas SVG con ejes, leyenda interactiva; tabla accesible equivalente (`<details>`) |
| `RooflinePlot` | `peak_flops`, `bw`, `points` | SVG log-log con techo computacional/memoria |
| `ScalingPlot` | `strong/weak: &'static [(u32,f64)]` | Speedup vs. hilos con línea ideal |
| `DiffusionSim` (refactor de `hpc.rs:23-27`) | `grid: RwSignal<Grid>`, `params` | Pasar el render de celdas a **`<canvas>` 2D** con `ImageData` (hoy SVG de 320×320 con N² `<rect>` es costoso); `requestAnimationFrame` con `on_cleanup`; botón *Pause*, `prefers-reduced-motion` → paso manual |
| `PostgisMapTile` | `geojson: &'static str`, `layers` | Proyección equirectangular a SVG (`<path>` desde GeoJSON simplificado) para parcelas AMYNTHAS; sin tiles externos |

### 4.4 Biología aplicada (`/field-research`)
| Componente | Props | Detalle |
|---|---|---|
| `TransectMap` | `plots: &'static [Plot]`, `metric: Signal<Metric>` | SVG con parcelas coloreadas por métrica (biomasa, pH, humedad) |
| `SpeciesAbundanceChart` | `obs: &'static [Obs]` | Barras apiladas / rank-abundance en escala log |
| `SoilProfile` | `layers: &'static [Horizon]` | Columna SVG de horizontes (O/A/B) con densidad de lombrices por capa |
| `ProtocolStepper` | `steps` | Lista numerada de protocolo de muestreo con `aria-current="step"` |
| `DataProvenance` | `dataset: &'static Dataset` | Tabla DOI/licencia/fecha/método/incertidumbre (cita y reproducibilidad) |

**Transversales**: `TechIcon { kind: Icon, size }` (SVG en *sprite* `<symbol>` + `<use>` para sustituir emojis y los 4 SVG duplicados de `home.rs:11-65`); `VisuallyHidden`; `Tooltip` (CSS puro con `:hover/:focus-visible`, reemplaza cualquier `onmouseover`); `Title`/`Meta` por ruta con `leptos_meta`.

---

## 5. Plan de refactorización de reactividad y CSS

### 5.1 Reactividad (Leptos 0.6)
1. **Separar escritura de lectura**: el task WS escribe en un `NodeStore`; la UI lee señales finas. Orden estable con `Vec<NodeId>` (insertar si no existe) en lugar de `HashMap::values()`.
2. **`<For>` con clave**: `<For each=move || store.ids.get() key=|id| id.clone() children=|id| view!{<NodeCard id/>}/>` → una tarjeta se crea una vez; solo cambia su señal `latest`.
3. **Granularidad de texto**: en `MetricReadout`, `{move || fmt(value.get())}`; el resto del DOM es estático. Los paquetes dejan de tocar >95 % del DOM.
4. **Aislar el reloj de *staleness***: eliminar `current_time` global. Dentro de `NodeCard`: `let stale = create_memo(move |_| now.get() - last_rx.get() > threshold)`, donde `now` es una señal compartida (`provide_context`) que actualiza un único `set_interval` a 1 s **o** a 5 s (el umbral es 5 s) — y solo `class:stale=stale` cambia, `Memo` evita notificar cuando no cambia el booleano. Mostrar la edad con `<AgeText last_rx/>` independiente.
5. **Eliminar clones**: `set_store.update(\|s\| { let n = s.entry(..).or_insert_with(..); n.ring.push(v); n.latest = payload; })`; o `RwSignal<RingBuffer>` por serie para no clonar `VecDeque`. `Show when` con `.with(\|m\| !m.is_empty())` o `create_memo`.
6. **Ciclo de vida**: sustituir `create_effect` + `set_interval_with_handle` por helper `use_interval(cb, dur)` con `on_cleanup`; para WS, usar un `AbortHandle` (`futures::future::AbortHandle`) o bandera `Rc<Cell<bool>>` comprobada en el bucle y cerrar el socket en `on_cleanup`. Mover el store a un **contexto raíz** (en `App`) para que la telemetría persista al cambiar de ruta y alimente nav/footer.
7. **Estados de conexión correctos**: pasar a `Connected` en `onopen`, usar `gloo_net` con `WebSocket::open` + primer mensaje o `web_sys::WebSocket` y `onopen`; *exponential backoff* (1→2→4→…→30 s) con *jitter* y cuenta atrás visible (`retry_in`).
8. **Fallo observable**: contadores `parse_errors`, `dropped`; `FetchState` para la DLQ; reintentar con *backoff* en polling; `AbortController` en peticiones.
9. **Arreglar P0** `:239` (ver 2.7); añadir expiración (`exp` del JWT) y botón "Bloquear".
10. **Chat**: `enum Sender`, `<For>` con id incremental, limpiar input al enviar, bloquear con `is_loading`, `cooldown` por 429 (leer `Retry-After`), auto-scroll con `NodeRef` + `create_effect`, `role="log"`.
11. **Un `api` módulo**: `fn api_base() -> Result<String>`; `fn ws_url()`; eliminar duplicación y *panics* (`expect`).
12. **Tamaño Wasm**: unificar cliente HTTP, perfil release (`opt-level="z"`, `lto="fat"`, `panic="abort"`), `data-wasm-opt="z"`.
13. **Sin inline JS ni estilos inline**: ya cumplido para JS; reemplazar `style=format!(...)` (`:110-112`) por clases `.series-temp/.series-co2` o `style:--series-color=…` (propiedad CSS, no atributo `style` completo) y `fill: var(--series-color)` en CSS.

### 5.2 CSS
1. **Capas**: `@layer reset, tokens, base, layout, components, pages, utilities;` en un `main.css` que importe parciales; Trunk puede enlazar varios `<link data-trunk rel="css">`.
2. **Tokens en 3 niveles**
   - *Primitivos*: `--gruvbox-bg0-h`, `--gruvbox-red-bright`…
   - *Semánticos globales*: `--surface-0/1/2`, `--text-primary/secondary/disabled`, `--border-subtle/strong` (≥3:1), `--focus-ring`.
   - *Semánticos SCADA*: `--state-ok/warn/fault/stale/calibrating/offline` + `--state-*-bg` (con `color-mix(in srgb, var(--state-fault) 15%, transparent)`) + `--series-1…8`.
   Reasignar `.metric-*` (por métrica) a clases de **estado** (`.is-ok/.is-warn/.is-fault`) y usar color de serie solo en la gráfica.
3. **Contraste**: usar `--accent-red-dim`/variante más clara para texto sobre `--bg-card` (≥4.5:1), `--border-strong` ≥3:1 para controles, no usar `opacity` para estado *stale*.
4. **Tipografía**: escala `--text-2xs…--text-3xl` (ratio 1.2), `--font-numeric` con `tabular-nums slashed-zero`; regla global para `td, output, time, .readout`.
5. **Auto-hospedar fuentes** y eliminar `index.html:13`; `font-display: swap`; subset latino.
6. **Eliminar CSS muerto** (`.console-shell`, `.telemetry-grid`, `.metric-row`, `.metric-value`).
7. **Movimiento**: agrupar animaciones en `@media (prefers-reduced-motion: no-preference)` (opt-in) en lugar de neutralizar después; añadir `.live-dot` a la lista.
8. **A11y**: `color-scheme: dark`; `:focus-visible` también para inputs (en lugar de `:focus`), tamaño mínimo de objetivo 24 px, `forced-colors`/`prefers-contrast: more` (aumenta bordes), `@media print`.
9. **Responsivo**: *container queries* para `NodeCard`; chatbot `width: min(360px, 100vw - 2rem)`.
10. **Lint CSS**: Stylelint (`stylelint-config-standard`) en CI opcional.

---

## 6. Roadmap priorizado (compatible con `cargo clippy -- -D warnings`)

> Cada paso es un PR pequeño; tras cada uno: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `trunk build`, prueba manual con backend local. Evitar `unwrap/expect` (el workspace aplica lints), `clippy::cast_precision_loss` (añadir `#[allow]` local como en `:40`), `needless_pass_by_value` en props (`#[prop(into)]` o `Signal`).

### P0 — Correctitud y resiliencia (primero)
1. **Corregir `room_monitoring.rs:239`**: enviar el esquema `Bearer` seguido del JWT (`token`) en `Authorization`; prueba manual: desbloquear → la tabla persiste >10 s.
2. **Distinguir "vacío" de "fallo" en DLQ** (`FetchState`), mensaje de error visible con `role="alert"`; botón "Bloquear".
3. **Eliminar `current_time` del closure de render** (`:399-400`): extraer `NodeCard` con `Memo` de staleness; `<For>` con clave y orden estable (`Vec<String>`).
4. **Detener WS/intervalos al desmontar** (`AbortHandle` + `on_cleanup`); marcar `Connected` solo en `onopen`; *backoff* exponencial.
5. **Mostrar `status` y `system_error_bitmask`** en cabecera de nodo (LED real + decodificador) para que un `HardwareError` sea visible; reemplazar emojis `:434,445` por SVG.
6. **Chat**: bloquear doble envío, limpiar input al enviar, manejo de 429 con cuenta atrás, `role="log"`/`aria-live`.
7. **Footer y `LIVE` reales** (`app.rs:54`, `nav.rs:41`) vía contexto `ConnectionStatus`.

### P1 — Cobertura de datos y sistema de diseño
8. Extraer `NodeStore`/`RingBuffer` a `frontend/src/state/` (sin clones) y mover URLs a `frontend/src/api.rs`.
9. Añadir readouts faltantes: `bvoc`, `tvoc`, `pm1_0`, conteos (`PmSizeBars`), `is_pm_out_of_range`, `laser_runtime`, `node_sequence`/`gateway_id`/versions.
10. `SensorSpec` + umbrales + estados `.is-ok/warn/fault`; sparklines para todas las métricas continuas; `TrendDelta`.
11. Backend + UI de `GatewayHealthEvent` (nuevo mensaje WS tipado o endpoint) → `GatewayHealthPanel`.
12. CSS: introducir `@layer` y tokens semánticos SCADA sin cambiar la apariencia; eliminar CSS muerto; corregir contraste (`--accent-red` en `--bg-card`, `--border-color`), `color-scheme`, `:focus-visible` en inputs; etiquetas `<label>` en inputs.
13. Self-host de fuentes; `lang="es"`; `leptos_meta::Title` por ruta; *skip-link*.
14. Perfil release/`wasm-opt`; unificar cliente HTTP; revisar dependencias no usadas.

### P2 — Narrativa profesional y demostradores
15. `/hardware`: `InstrumentSheet`, `BomTable`, `SchematicViewer`, `EnclosureViewer`, `PowerBudget`.
16. `/hpc`: `BenchmarkChart` (AVX-512 vs AVX2 vs escalar), `RooflinePlot`, `ScalingPlot`; `DiffusionSim` en `<canvas>` + rAF; `PostgisMapTile`.
17. `/field-research`: `TransectMap`, `SpeciesAbundanceChart`, `SoilProfile`, `DataProvenance`.
18. Home: tarjetas con métricas en vivo (nº de nodos, última lectura), sección de publicaciones, `og:image`, `@media print`.
19. Vista de flota/detalle (`/telemetry/room/:device_id`) con histórico ampliado (120–600 puntos, Canvas 2D) y exportación CSV.
20. Pruebas: `wasm-bindgen-test` para ring buffer, decodificador de bitmask y detector de huecos; CI con `trunk build --release` + Lighthouse (a11y ≥ 95).
