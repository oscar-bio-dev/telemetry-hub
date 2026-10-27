# Auditoría GUI v2 — Evaluación de mejoras, paleta, legibilidad y experiencia

> Solo análisis. Base evaluada: `origin/main` @ `60ce8ff` (Sprint 1 P0: `da3725f`, `aad6205`) frente a la v1 (`docs/GUI_AUDIT_REPORT.md`, commit `da84b41`).
> Los contrastes se calcularon con la fórmula WCAG 2.x. No se compiló el Wasm ni se midió en navegador: las observaciones de "sensación" son **inferencias del código y del CSS**, no pruebas con usuarios.

## 1. Veredicto sobre las mejoras

**Muy buen avance en fiabilidad, avance nulo en cobertura de datos y en diseño visual.** Sprint 1 resolvió casi todo lo P0 de la v1.

| P0 de la v1 | Estado en `main` | Evidencia |
|---|---|---|
| Bug `Authorization` de la DLQ | ✅ Corregido (`Bearer`) | `room_monitoring.rs` ~l.546-550 |
| "Vacío" vs "fallo" en DLQ | ✅ `DlqFetchState {Idle,Loading,Ok,Err}` + `role="alert"` | `types.rs:22-28` |
| Re-render global por `current_time` | ✅ `NodeCard` con memos; el tick solo toca `is_stale`/`age_text` | `room_monitoring.rs:151-185` |
| Orden no determinista de tarjetas | ✅ `device_ids` ordenado + `<For key>` | `room_monitoring.rs:495-502` |
| WS: backoff, cancelación, parse errors | ✅ backoff 1→30 s, bandera de cancelación, contador visible | `room_monitoring.rs` ~l.655-760 |
| Footer / LIVE falsos | ✅ contexto `ConnectionStatus` compartido | `app.rs:16-17,61`; `nav.rs:10-12` |
| Status / bitmask / `is_pm_out_of_range` invisibles | ✅ LED + texto de estado + alerta con bitmask | `room_monitoring.rs:211-300` |
| Emojis de alerta | ✅ SVG inline | `room_monitoring.rs:236-300` |
| Chat: 429, doble envío, `aria-live`, `enum Sender`, `<For>` | ✅ cooldown, `role="log"`, bloqueo de envío | `chatbot.rs:19-28,115,151-158,178` |
| URLs duplicadas / `expect` | ✅ `utils.rs` sin panics | `utils.rs:1-50` |
| `<label>` del input DLQ, botón Bloquear | ✅ | `room_monitoring.rs` ~l.380, 416-425 |

**Pendiente o nuevo:**
- `payload.get().map(...)` (`room_monitoring.rs:321`) reconstruye **toda** la rejilla de lecturas en cada paquete. La granularidad se logró a nivel de tarjeta, no de lectura.
- `temp_history`/`co2_history` se clonan en el memo (`:203-210`) y otra vez en el worker WS (`:686-689`).
- Cobertura de datos: siguen sin mostrarse `bvoc`, `tvoc`, `pm1_0`, los 3 conteos, `laser_runtime`, `node_sequence`, `gateway_id`; `GatewayHealthEvent` sigue sin usarse.
- Sparklines solo en 2 de 13 métricas; "Latencia Red" sigue midiendo nodo→gateway (`:473`).
- Aparece una inconsistencia nueva: `.alert-hardware-fault` y `.dlq-error-box` usan `rgba(204,36,29,…)`, mientras que `--status-fault` es `#fb4934`.
- `index.html` sin cambios: `lang="en"` y Google Fonts externas.
- El botón SEND no tiene estilo `:disabled` ni de cooldown (no hay ninguna regla `disabled` en el CSS). El texto cambia a `WAIT (12s)`, pero visualmente el botón sigue pareciendo activo.
- El cooldown del chat es fijo (12 s) y un `set_timeout` independiente lo pone a 0; no se lee `Retry-After`.

**Notas actualizadas (1-10):** datos 4→**4**; arquitectura de información **7**; reactividad 4→**7**; CSS/a11y 6→**6,5**; seguridad/resiliencia 3→**8**. Media 4,8→**6,5**.

## 2. Paleta de colores (Gruvbox Hard Dark)

### 2.1 Contraste medido

| Color | Hex | sobre `--bg-hard` #1d2021 | sobre `--bg-card` #32302f |
|---|---|---|---|
| `--fg-main` | #ebdbb2 | 11,95 | 9,57 |
| `--fg-muted` | #a89984 | 5,90 | 4,72 |
| `--accent-yellow` | #fabd2f | 9,67 | 7,74 |
| `--accent-green` | #b8bb26 | 7,94 | 6,36 |
| `--accent-orange` | #fe8019 | 6,49 | 5,20 |
| `--accent-purple` | #d3869b | 5,98 | 4,78 |
| `--accent-blue` | #83a598 | 6,09 | 4,88 |
| `--accent-red` | #fb4934 | 4,77 | **3,82** ✘ |
| `--border-color` | #504945 | **1,86** ✘ | **1,49** ✘ |
| texto rojo sobre la caja de alerta (`#fb4934` sobre rojo al 15 % en card, ≈ `#492e2c`) | | | **3,57** ✘ |

La paleta general es **cómoda y de alto contraste** (fondo cálido, no negro puro; texto crema ≈ 12:1). Es una buena elección para ambientes de poca luz o sala de control, y reduce la fatiga visual frente a blanco sobre negro.

### 2.2 Problemas de uso del color

1. **`.metric-readout-tolerance` usa `--border-color` como color de texto** (`style.css:1025-1030`): contraste **1,49:1** sobre la tarjeta. La incertidumbre (±0,5 °C, etc.) es el dato metrológico que da autoridad científica y es prácticamente ilegible; solo mejora al pasar el ratón (`:1032`). Es el hallazgo de legibilidad más grave de la UI.
2. **Colisión semántica rojo/amarillo/verde.** El color indica a la vez *tipo de métrica* y *estado*:
   - CO₂ es rojo (`:1061`) = el mismo `--status-fault` de una avería. Un CO₂ de 420 ppm normal se ve como alarma.
   - Amarillo es temperatura, PM2.5, eCO₂ y presión (4 métricas) y además el color de "advertencia", del botón del chat y del `badge-calibrating`.
   - Verde es IAQ y pH, y también "OK".
   El operador no puede saber por el color si un valor es bueno o malo; el color decora en vez de informar. Es el principio contrario al de un HMI/SCADA (ISA-101): color reservado para estado anormal, datos en tonos neutros.
3. **Cuatro tonos de acento sin jerarquía de datos.** 13 métricas con ~7 colores distintos crean una rejilla "de arcoíris" que compite con las alarmas. Con tantas lecturas, las anomalías no destacan.
4. **Rojo fijo en los tres puntos de la cabecera** (`.dot.red/yellow/green`, `style.css:142-148`; `room_monitoring.rs` los sigue usando en otras tarjetas): ahora conviven con el LED de estado real y se leen como un segundo "semáforo" falso. Con la v1 era decorativo; ahora es ruido contradictorio.
5. **`.stale-data` (`opacity: .5` + `grayscale(.8)`)** reduce el texto atenuado por debajo de 3:1. Los datos obsoletos pasan a ser difíciles de leer justo cuando el operador debe decidir si confía en ellos.
6. **Daltonismo (protanopia/deuteranopia):** rojo, naranja, amarillo y verde de Gruvbox colapsan en tonos de amarillo-oliva. El LED es la única señal de estado en la cabecera y comparte forma (círculo 8 px) entre OK/WARN/FAULT; lo salva el texto `MONITORING/…`. Es aceptable, pero convendría una forma distinta por estado. (No se simuló numéricamente; es una valoración cualitativa.)
7. **Resplandor (`box-shadow` 6-8 px) en LEDs** funciona en oscuro, pero el parpadeo `blink-fast` del fallo (0,8 s, infinito) es agresivo; está cubierto por `prefers-reduced-motion`, pero ver §4.

### 2.3 Recomendación
- Datos en **neutro** (`--fg-main`), cada serie con un acento único y apagado, y solo `ok/warn/fault` en color saturado.
- Crear `--state-*` y `--text-tertiary` (≥ 4,5:1, p. ej. ~`#9d8d79`) para tolerancias; `--border-strong` ≥ 3:1 (~`#7c6f64`) para controles; `--status-fault-text` más claro (~`#ff6a55`) para texto sobre card.
- Cambiar el color del CO₂ a azul/neutro y pintar el valor en rojo solo por encima del umbral (1000/1500 ppm).

## 3. Comportamiento del usuario y sensación al navegar

**Escenarios y recorrido probable**

1. **Reclutador / ingeniero (10-60 s en la home).** Llega a la terminal estilizada, `oscar@bio-dev:~/telemetry-hub$ ./initialize_scada_hmi`: identidad fuerte y coherente con el perfil técnico. La nav con prefijos `~/` refuerza la narrativa. Es memorable y diferencia el portafolio. Riesgo: el prompt decorativo y el subtítulo largo ocupan el primer viewport antes de que aparezca **ninguna prueba** (un instrumento, un número, una imagen). No hay fotos del hardware, ni esquemáticos ni gráficos en la home.
2. **Operador (monitoreo continuo).** Aquí las mejoras se notan: el DOM de las tarjetas ya no parpadea, el foco y la selección de texto sobreviven, el LED y el texto de estado son claros, el estado de conexión es coherente en nav, banner y footer. La sensación es de "herramienta estable". Lo que aún se siente tosco: los valores se actualizan sin ninguna señal de cambio (sin flash, sin delta ni flecha de tendencia), y el reintento WS muestra `Disconnected` sin cuenta atrás visible.
3. **Operador con un incidente.** Ve un banner rojo con el bitmask hexadecimal (`0x00A4`) **sin traducción** a una causa; hay que ir a la documentación para interpretarlo. Es información, no diagnóstico.
4. **Usuario con el chat.** Al recibir un 429 hay mensaje de sistema y `WAIT (Ns)`: bien. Falta guiar con el estado visual (botón atenuado), mostrar el contador en el input y sugerir preguntas iniciales; el asistente no conoce el nodo que se está mirando.

**Sensación general (inferida):** profesional, "de ingeniero para ingenieros", sobria. La metáfora de terminal + tarjetas con tres puntos es coherente pero **repetitiva**: cada panel (home, hardware, hpc, telemetría, DLQ, 404) usa el mismo marco de ventana con tres puntos, sin jerarquía; todo pesa lo mismo. En telemetría, 13 lecturas × N nodos × bordes redondeados + sombras producen una pared de tarjetas que obliga a escanear, sin un resumen de "¿está todo bien?" arriba.

**Fricciones concretas de navegación**
- No hay vista de resumen de flota ni filtro/orden: con varios nodos, hay que hacer scroll vertical largo para comparar.
- Sin migas ni enlace "volver" salvo la nav; la nav no se fija (sticky) al hacer scroll en la página larga de telemetría (no hay `position: sticky` en `.terminal-nav`; verificar).
- Cambio de ruta: el store WS vive en `RoomMonitoring`; el estado de conexión es global pero **el historial de sparklines se pierde al navegar fuera y volver**, y la sensación es de "empezar de cero" cada vez.
- No hay estado de carga al arrancar el Wasm (pantalla en blanco inicial; `index.html` sin splash ni `<noscript>`), que penaliza la primera impresión en conexiones lentas.
- Ningún atajo de teclado (p. ej. `/` para el chat, `Esc` para cerrarlo); el chat no gestiona foco ni `Escape`.
- Interacción táctil/móvil: `chatbot-window` de 360 px fijos (`style.css:591`) desborda en pantallas de 320-360 px; `.node-status-text` 0,72 rem y las tolerancias 0,7 rem quedan muy pequeñas en móvil.

## 4. Legibilidad

| Aspecto | Evaluación |
|---|---|
| Valor numérico (1,8 rem mono, `tabular-nums`) | ✅ Muy legible, no "baila" |
| Etiqueta en mayúsculas 0,75 rem, `letter-spacing` 0,6 px | ◐ Aceptable en desktop; justo en móvil |
| **Tolerancia 0,7 rem (11,2 px) a 1,49:1** | ❌ Ilegible; se debe subir a ≥ 0,75 rem y ≥ 4,5:1 |
| Cuerpo en `Inter` y todo lo demás en `Fira Code` | ◐ Mucho monoespaciado en prosa (`.section-desc`, `.executive-statement`): cansa en párrafos largos; reservar mono a datos/IDs y usar sans para texto corrido |
| 15 usos de `font-size: 0.7x rem` y 4 de `0.5-0.6 rem` | ◐ Escala ad hoc; muchas piezas por debajo de 12 px |
| Mezcla de español e inglés (`Live Data (0s)`, `Stale: actualizado hace…`, `SEND`, `ESTABLISHING LINK`, `Parse Errors`) | ◐ Inconsistente; y `lang="en"` en `index.html` hace que el lector de pantalla lea el español con fonética inglesa |
| Longitud de línea | ✅ `max-width: 1320px` con rejillas; el texto corrido de la home no excede la medida cómoda en las tarjetas |
| Contraste de texto de cuerpo (11,95:1) | ✅ Excelente |
| Contraste de texto atenuado (4,7-5,9:1) | ✅ Pasa AA (justo en card) |
| Cursor/`blink`, pulsos infinitos | ◐ Cubiertos por `prefers-reduced-motion`, pero `pulse-text` desactivado deja "ESTABLISHING LINK…" sin indicio de actividad |
| Tabla DLQ con Base64 en celda 220 px (`word-break: break-all`) | ◐ Legible pero sin truncado ni botón copiar |

## 5. Experiencia: lo que funciona y lo que falta

**Fortalezas:** identidad visual única y consistente; estabilidad percibida; estados de conexión y error veraces; accesibilidad por encima de la media (`:focus-visible`, `.sr-only`, `role="alert"`, `role="log"`, labels, `scope="col"`); la sparkline y los readouts industriales con tolerancias dan seriedad metrológica *(cuando se pueden leer)*.

**Brechas principales de experiencia**
1. **Sin "jerarquía de lectura" en telemetría:** no hay un resumen de salud de flota (N nodos OK / WARN / FAULT / STALE) antes de las tarjetas.
2. **Color con doble función** (§2.2): bloquea la lectura rápida de anomalías.
3. **Datos sin contexto:** valor sin umbral, sin Δ, sin rango operativo, sin historial salvo 2 métricas.
4. **Narrativa de portafolio sin evidencias visuales** en la home (fotos, esquemáticos, benchmark, mapa).
5. **Retroalimentación débil de la acción:** botón SEND sin estado deshabilitado, DLQ sin indicador de "última actualización", datos que cambian sin microanimación sutil.
6. **Documentación del dominio dentro de la UI:** el bitmask y los códigos de estado no se explican.

## 6. Prioridades recomendadas (nuevas, ordenadas por impacto en UX)

**Quick wins (CSS, bajo riesgo, sin tocar Rust)**
1. `.metric-readout-tolerance` → `var(--fg-muted)` y `font-size: .75rem` (de 1,49:1 a 4,72:1).
2. Añadir `.chatbot-btn-send:disabled` y estado `.is-cooling` (opacidad + `cursor: not-allowed`).
3. Unificar los `rgba(204,36,29,…)` con `--status-fault` mediante `color-mix(in srgb, var(--status-fault) 15%, transparent)`.
4. `color-scheme: dark`; chatbot `width: min(360px, calc(100vw - 2rem))`; `.terminal-nav { position: sticky; top: 0 }` si procede.
5. `.stale-data`: atenuar solo valores y añadir etiqueta `STALE`, no `opacity` global.

**Rust pequeño**
6. `lang="es"`, `<noscript>`, indicador de carga inicial en `index.html`; auto-hospedar fuentes.
7. Mover el color de las métricas a neutro y aplicar clases `is-ok/is-warn/is-fault` según umbrales.
8. Panel "Resumen de flota" arriba del grid (conteo por estado) y decodificador del bitmask.
9. Pasar `payload.get().map(...)` (`:321`) a lecturas con señal propia (`MetricReadout` con `Signal<Option<f64>>`) y reemplazar los `.clone()` del historial por un buffer circular.
10. Cuenta atrás visible de reconexión y lectura de `Retry-After` en el chat.

**Mediano plazo (ya listado en v1 §4-§6):** cobertura de los 15 campos restantes, `GatewayHealthPanel`, evidencias visuales en home/hardware/hpc/field-research, vista de detalle por nodo con histórico ampliado.

## 7. Resumen en una línea
La **robustez** pasó de frágil a sólida; la **experiencia visual** sigue siendo la de la v1: una paleta bonita y cómoda que aún usa el color como decoración (CO₂ siempre rojo, 4 métricas amarillas), con el dato metrológico más valioso (tolerancia) casi invisible y sin jerarquía de lectura rápida para el operador.
