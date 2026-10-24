// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.

use crate::components::spec_sheet::{SpecRow, SpecSheet};
use leptos::*;

/// Diagrama de bloques SVG: EcoWest WQM-V100 (Sonda Óptica DO y Electrónica Front-End)
#[component]
fn DiagramWqmV100() -> impl IntoView {
    view! {
        <div class="terminal-card diagram-card">
            <div class="card-header diagram-header">
                <span class="dot red" aria-hidden="true"></span>
                <span class="dot yellow" aria-hidden="true"></span>
                <span class="dot green" aria-hidden="true"></span>
                <span class="filename">"schematic_blocks::eco_west_wqm_v100.svg"</span>
            </div>
            <div class="card-body diagram-body">
                <svg class="block-diagram-svg" viewBox="0 0 760 140" fill="none" xmlns="http://www.w3.org/2000/svg" aria-label="Diagrama de Bloques WQM-V100">
                    // Bloque 1: Excitación Óptica
                    <rect x="10" y="30" width="120" height="70" rx="4" class="diag-block block-accent-blue" />
                    <text x="70" y="58" class="diag-text-title" text-anchor="middle">"Excitador LED"</text>
                    <text x="70" y="76" class="diag-text-sub" text-anchor="middle">"λ = 470 nm (Azul)"</text>
                    <text x="70" y="90" class="diag-text-detail" text-anchor="middle">"Modulación 5 kHz"</text>

                    // Flecha 1 -> 2
                    <path d="M130 65 L160 65" class="diag-arrow" />
                    <polygon points="160,61 168,65 160,69" class="diag-arrowhead" />

                    // Bloque 2: Cámara Sensora y Fluoróforo
                    <rect x="168" y="30" width="130" height="70" rx="4" class="diag-block block-accent-green" />
                    <text x="233" y="58" class="diag-text-title" text-anchor="middle">"Matriz Fluorófora"</text>
                    <text x="233" y="76" class="diag-text-sub" text-anchor="middle">"Luminiscencia Ru/Pt"</text>
                    <text x="233" y="90" class="diag-text-detail" text-anchor="middle">"Quenching por O2"</text>

                    // Flecha 2 -> 3
                    <path d="M298 65 L328 65" class="diag-arrow" />
                    <polygon points="328,61 336,65 328,69" class="diag-arrowhead" />

                    // Bloque 3: Fotodiodo y TIA
                    <rect x="336" y="30" width="125" height="70" rx="4" class="diag-block block-accent-yellow" />
                    <text x="398" y="58" class="diag-text-title" text-anchor="middle">"Fotodiodo PIN + TIA"</text>
                    <text x="398" y="76" class="diag-text-sub" text-anchor="middle">"λ = 650 nm (Rojo)"</text>
                    <text x="398" y="90" class="diag-text-detail" text-anchor="middle">"Filtro Óptico 600nm+"</text>

                    // Flecha 3 -> 4
                    <path d="M461 65 L491 65" class="diag-arrow" />
                    <polygon points="491,61 499,65 491,69" class="diag-arrowhead" />

                    // Bloque 4: Procesamiento MCU Core
                    <rect x="499" y="20" width="125" height="90" rx="4" class="diag-block block-accent-main" />
                    <text x="561" y="48" class="diag-text-title" text-anchor="middle">"MCU TI MSPM0"</text>
                    <text x="561" y="66" class="diag-text-sub" text-anchor="middle">"ADC 24-bit / DSP"</text>
                    <text x="561" y="82" class="diag-text-detail" text-anchor="middle">"Δθ Fase -> DO mg/L"</text>
                    <text x="561" y="96" class="diag-text-detail" text-anchor="middle">"Comp. Temp & Presión"</text>

                    // Flechas dobles de salida de MCU
                    <path d="M624 45 L654 45" class="diag-arrow" />
                    <polygon points="654,41 662,45 654,49" class="diag-arrowhead" />
                    <path d="M624 85 L654 85" class="diag-arrow" />
                    <polygon points="654,81 662,85 654,89" class="diag-arrowhead" />

                    // Bloque 5a: Display
                    <rect x="662" y="25" width="90" height="38" rx="3" class="diag-block block-sub" />
                    <text x="707" y="42" class="diag-text-title" text-anchor="middle">"SHARP LCD"</text>
                    <text x="707" y="55" class="diag-text-detail" text-anchor="middle">"Memory 128x64"</text>

                    // Bloque 5b: RS485 Modbus
                    <rect x="662" y="75" width="90" height="38" rx="3" class="diag-block block-sub" />
                    <text x="707" y="92" class="diag-text-title" text-anchor="middle">"RS485 Bus"</text>
                    <text x="707" y="105" class="diag-text-detail" text-anchor="middle">"Modbus RTU"</text>
                </svg>
            </div>
        </div>
    }
}

/// Diagrama de bloques SVG: Gateway Dual-Chip ESP32-P4 / C6
#[component]
fn DiagramGateway() -> impl IntoView {
    view! {
        <div class="terminal-card diagram-card">
            <div class="card-header diagram-header">
                <span class="dot red" aria-hidden="true"></span>
                <span class="dot yellow" aria-hidden="true"></span>
                <span class="dot green" aria-hidden="true"></span>
                <span class="filename">"schematic_blocks::esp32_p4_c6_gateway.svg"</span>
            </div>
            <div class="card-body diagram-body">
                <svg class="block-diagram-svg" viewBox="0 0 760 140" fill="none" xmlns="http://www.w3.org/2000/svg" aria-label="Diagrama de Bloques Gateway Dual-Chip">
                    // Bloque 1: Nodos de Campo
                    <rect x="10" y="30" width="125" height="70" rx="4" class="diag-block block-accent-green" />
                    <text x="72" y="58" class="diag-text-title" text-anchor="middle">"Nodos Sensores"</text>
                    <text x="72" y="76" class="diag-text-sub" text-anchor="middle">"ESP-NOW 2.4 GHz"</text>
                    <text x="72" y="90" class="diag-text-detail" text-anchor="middle">"Protobuf Nanopb"</text>

                    // Flecha 1 -> 2
                    <path d="M135 65 L165 65" class="diag-arrow" />
                    <polygon points="165,61 173,65 165,69" class="diag-arrowhead" />

                    // Bloque 2: Front-End RF (ESP32-C6)
                    <rect x="173" y="30" width="130" height="70" rx="4" class="diag-block block-accent-blue" />
                    <text x="238" y="58" class="diag-text-title" text-anchor="middle">"ESP32-C6 (RF Edge)"</text>
                    <text x="238" y="76" class="diag-text-sub" text-anchor="middle">"Wi-Fi 6 + BLE 5 + 802.15.4"</text>
                    <text x="238" y="90" class="diag-text-detail" text-anchor="middle">"Desempaquetado MAC"</text>

                    // Flecha 2 -> 3 (SPI Bus de Alta Velocidad)
                    <path d="M303 65 L333 65" class="diag-arrow" />
                    <polygon points="333,61 341,65 333,69" class="diag-arrowhead" />

                    // Bloque 3: Procesador Host Dual RISC-V (ESP32-P4)
                    <rect x="341" y="20" width="155" height="90" rx="4" class="diag-block block-accent-main" />
                    <text x="418" y="46" class="diag-text-title" text-anchor="middle">"ESP32-P4 Core"</text>
                    <text x="418" y="64" class="diag-text-sub" text-anchor="middle">"Dual RISC-V @ 400 MHz"</text>
                    <text x="418" y="80" class="diag-text-detail" text-anchor="middle">"Acelerador ECDSA P-256"</text>
                    <text x="418" y="94" class="diag-text-detail" text-anchor="middle">"Firma JWT y Buffer RAM"</text>

                    // Flechas de bifurcación (MicroSD y Backhaul)
                    <path d="M496 45 L526 45" class="diag-arrow" />
                    <polygon points="526,41 534,45 526,49" class="diag-arrowhead" />
                    <path d="M496 85 L526 85" class="diag-arrow" />
                    <polygon points="526,81 534,85 526,89" class="diag-arrowhead" />

                    // Bloque 4a: Buffer Circular
                    <rect x="534" y="25" width="105" height="40" rx="3" class="diag-block block-accent-yellow" />
                    <text x="586" y="43" class="diag-text-title" text-anchor="middle">"MicroSD SLC"</text>
                    <text x="586" y="56" class="diag-text-detail" text-anchor="middle">"Buffer Circular Offline"</text>

                    // Bloque 4b: Módems Backhaul
                    <rect x="534" y="75" width="105" height="40" rx="3" class="diag-block block-accent-blue" />
                    <text x="586" y="93" class="diag-text-title" text-anchor="middle">"Ethernet / LTE-M"</text>
                    <text x="586" y="106" class="diag-text-detail" text-anchor="middle">"Enlace Encriptado TLS"</text>

                    // Flecha 4b -> 5
                    <path d="M639 95 L669 95" class="diag-arrow" />
                    <polygon points="669,91 677,95 669,99" class="diag-arrowhead" />

                    // Bloque 5: Cloud
                    <rect x="677" y="70" width="75" height="50" rx="4" class="diag-block block-accent-green" />
                    <text x="714" y="92" class="diag-text-title" text-anchor="middle">"GCP Cloud"</text>
                    <text x="714" y="106" class="diag-text-detail" text-anchor="middle">"Pub/Sub"</text>
                </svg>
            </div>
        </div>
    }
}

/// Diagrama de bloques SVG: Nodos Sensores Ambientales (BME688 / SCD41 / BMV080)
#[component]
fn DiagramSensorNode() -> impl IntoView {
    view! {
        <div class="terminal-card diagram-card">
            <div class="card-header diagram-header">
                <span class="dot red" aria-hidden="true"></span>
                <span class="dot yellow" aria-hidden="true"></span>
                <span class="dot green" aria-hidden="true"></span>
                <span class="filename">"schematic_blocks::environmental_sensor_array.svg"</span>
            </div>
            <div class="card-body diagram-body">
                <svg class="block-diagram-svg" viewBox="0 0 760 140" fill="none" xmlns="http://www.w3.org/2000/svg" aria-label="Diagrama de Bloques Sensor Multiparamétrico">
                    // Sensores a la izquierda
                    <rect x="10" y="15" width="150" height="32" rx="3" class="diag-block block-accent-yellow" />
                    <text x="85" y="31" class="diag-text-title" text-anchor="middle">"Bosch BME688"</text>
                    <text x="85" y="42" class="diag-text-detail" text-anchor="middle">"IAQ / Gas VOC / Temp / P / RH"</text>

                    <rect x="10" y="55" width="150" height="32" rx="3" class="diag-block block-accent-red" />
                    <text x="85" y="71" class="diag-text-title" text-anchor="middle">"Sensirion SCD41"</text>
                    <text x="85" y="82" class="diag-text-detail" text-anchor="middle">"Fotoacústico NDIR CO2"</text>

                    <rect x="10" y="95" width="150" height="32" rx="3" class="diag-block block-accent-yellow" />
                    <text x="85" y="111" class="diag-text-title" text-anchor="middle">"Bosch BMV080"</text>
                    <text x="85" y="122" class="diag-text-detail" text-anchor="middle">"Láser PM1.0 / PM2.5 / PM10"</text>

                    // Buses de comunicación
                    <path d="M160 31 L230 31 L230 60 L260 60" class="diag-arrow" />
                    <path d="M160 71 L260 71" class="diag-arrow" />
                    <path d="M160 111 L230 111 L230 82 L260 82" class="diag-arrow" />
                    <polygon points="260,67 268,71 260,75" class="diag-arrowhead" />

                    // Bloque Central: Arm Cortex-M23 / MSPM0
                    <rect x="268" y="20" width="165" height="95" rx="4" class="diag-block block-accent-main" />
                    <text x="350" y="46" class="diag-text-title" text-anchor="middle">"Ultra-Low-Power MCU"</text>
                    <text x="350" y="64" class="diag-text-sub" text-anchor="middle">"PIC32CM / TI MSPM0"</text>
                    <text x="350" y="80" class="diag-text-detail" text-anchor="middle">"BSEC Library Compilada"</text>
                    <text x="350" y="94" class="diag-text-detail" text-anchor="middle">"Nanopb Mega-Schema v2.0"</text>
                    <text x="350" y="106" class="diag-text-detail" text-anchor="middle">"Consumo: 1.5 µA Sleep"</text>

                    // Flecha Central -> Transmisor RF
                    <path d="M433 67 L480 67" class="diag-arrow" />
                    <polygon points="480,63 488,67 480,71" class="diag-arrowhead" />

                    // Transmisor ESP-NOW
                    <rect x="488" y="35" width="135" height="65" rx="4" class="diag-block block-accent-blue" />
                    <text x="555" y="62" class="diag-text-title" text-anchor="middle">"Transmisor RF"</text>
                    <text x="555" y="78" class="diag-text-sub" text-anchor="middle">"ESP-NOW / LoRaWAN"</text>
                    <text x="555" y="91" class="diag-text-detail" text-anchor="middle">"+20 dBm EIRP"</text>

                    // Flecha RF -> Antena
                    <path d="M623 67 L665 67" class="diag-arrow" />
                    <polygon points="665,63 673,67 665,71" class="diag-arrowhead" />

                    // Antena
                    <rect x="673" y="45" width="75" height="45" rx="3" class="diag-block block-accent-green" />
                    <text x="710" y="67" class="diag-text-title" text-anchor="middle">"Antena"</text>
                    <text x="710" y="81" class="diag-text-detail" text-anchor="middle">"Omni 2.4 GHz"</text>
                </svg>
            </div>
        </div>
    }
}

#[component]
pub fn HardwarePage() -> impl IntoView {
    let wqm_specs = vec![
        SpecRow::new(
            "Principio de Medición",
            "Luminiscencia Óptica (Decaimiento de Fase)",
            "Fluorometría",
            "0.0 - 20.0 mg/L",
            "Sin consumo de O2, cero deriva",
        ),
        SpecRow::new(
            "Resolución DO",
            "0.01",
            "mg/L",
            "±0.1 mg/L o ±1%",
            "Compensación automática temp/salinidad",
        ),
        SpecRow::new(
            "Rango de Temperatura",
            "-10.0 a +60.0",
            "°C",
            "±0.1 °C (PT1000 Class A)",
            "Sensor embebido en sonda de titanio",
        ),
        SpecRow::new(
            "Tiempo de Respuesta (t90)",
            "< 30",
            "segundos",
            "Flujo estático",
            "Membrana fluorófora reemplazable",
        ),
        SpecRow::new(
            "Display Integrado",
            "SHARP Memory LCD 128x64",
            "Reflectivo monocromo",
            "Ultra-bajo consumo (15 µW)",
            "Lectura clara bajo luz solar directa",
        ),
        SpecRow::new(
            "Compensación Barométrica",
            "SCD41 / BME688 Integrado",
            "hPa",
            "300 a 1100 hPa (±0.6 hPa)",
            "Cálculo barométrico automático en vivo",
        ),
        SpecRow::new(
            "Interfaz de Comunicación",
            "RS485 Modbus RTU / UART",
            "Baud 9600-115200",
            "Aislamiento galvánico 2.5 kV",
            "Direccionamiento configurable",
        ),
        SpecRow::new(
            "Alimentación y Consumo",
            "5.0 - 24.0 VDC",
            "mA",
            "12 mA activo, 0.05 mA sleep",
            "Optimizado para baterías solares LiFePO4",
        ),
        SpecRow::new(
            "Protección Mecánica",
            "IP68 / Titanio Grado 2 & POM",
            "Sumergible",
            "Hasta 10 bar (100 m)",
            "Resistente a ambientes salinos y sulfuros",
        ),
    ];

    let gateway_specs = vec![
        SpecRow::new(
            "Procesador Principal",
            "Espressif ESP32-P4 Dual-Core RISC-V",
            "400 MHz",
            "FPU + Cripto Hardware",
            "Coprocesador criptográfico para firmas ECDSA P-256",
        ),
        SpecRow::new(
            "Front-End Inalámbrico",
            "Espressif ESP32-C6 RISC-V",
            "160 MHz",
            "Wi-Fi 6 + BLE 5 + 802.15.4",
            "Recepción simultánea ESP-NOW y Thread",
        ),
        SpecRow::new(
            "Formato de Enlace Radio",
            "Google Protocol Buffers (Nanopb)",
            "Binario compacto",
            "33 campos canónicos",
            "Deserialización en < 10 µs en el gateway",
        ),
        SpecRow::new(
            "Seguridad de Enlace",
            "mTLS + JWT ES256",
            "Criptografía asimétrica",
            "Firmas ECDSA P-256",
            "Autenticación zero-trust hacia GCP Pub/Sub",
        ),
        SpecRow::new(
            "Almacenamiento Local",
            "MicroSD Industrial UHS-I",
            "GB",
            "32 GB SLC Mode",
            "Buffer circular persistente ante cortes de red",
        ),
        SpecRow::new(
            "Conectividad Backhaul",
            "Ethernet 10/100 + LTE-M / NB-IoT",
            "Cat-M1",
            "Banda global B1-B28",
            "Fallback automático Ethernet -> Celular",
        ),
    ];

    let sensor_node_specs = vec![
        SpecRow::new(
            "Microcontrolador Central",
            "Microchip PIC32CM LE00 / TI MSPM0",
            "Arm Cortex-M23",
            "48 MHz, 128KB Flash",
            "Ultra-low power sleep 1.5 µA",
        ),
        SpecRow::new(
            "Sensor CO2 NDIR",
            "Sensirion SCD41 Fotoacústico",
            "ppm",
            "400 - 5000 ppm (±40 ppm + 5%)",
            "Compensación de presión barométrica en vivo",
        ),
        SpecRow::new(
            "Calidad de Aire & VOCs",
            "Bosch Sensortec BME688",
            "IAQ / Gas Res",
            "Índice 0-500, Gas 0-50 MΩ",
            "BSEC library embebida en nodo",
        ),
        SpecRow::new(
            "Material Particulado",
            "Bosch Sensortec BMV080",
            "µg/m³",
            "PM1.0 / PM2.5 / PM10 (±10%)",
            "Láser óptico sin ventilador",
        ),
        SpecRow::new(
            "Calibración y Diagnóstico",
            "Warmup BSEC & Obstrucción Óptica",
            "Diagnóstico activo",
            "12 pulsos iniciales",
            "Alerta automática en telemetría ante lente sucia",
        ),
        SpecRow::new(
            "Transmisión Inalámbrica",
            "ESP-NOW / LoRaWAN Sub-GHz",
            "dBm",
            "+20 dBm EIRP",
            "Topología en malla / estrella para campo",
        ),
        SpecRow::new(
            "Autonomía Energética",
            "Célula LiFePO4 3.2V 3200mAh + Panel 2W",
            "Años",
            "> 3 años sin mantenimiento",
            "Ciclo duty-cycle adaptativo por nivel de batería",
        ),
    ];

    view! {
        <div class="page-container hardware-page">
            <header class="section-header">
                <h2 class="section-title">"~/hardware - Instrumentación y Sensores Bare-Metal"</h2>
                <p class="section-desc">
                    "Documentación técnica de arquitectura de hardware, transductores ópticos y gateways industriales desarrollados por Oscar Mora en EcoWest Instrument."
                </p>
            </header>

            <div class="hardware-stack">
                // Sistema 1: EcoWest WQM-V100
                <article class="hardware-system-entry">
                    <h3 class="system-entry-title">"1. EcoWest WQM-V100 (Medidor Óptico de Oxígeno Disuelto)"</h3>
                    <p class="system-entry-desc">
                        "Instrumento oceanográfico y de acuicultura basado en fotometría de decaimiento luminiscente de fase. Integra un microcontrolador TI MSPM0 / PIC32CM con pantalla reflectiva SHARP Memory LCD para visualización en campo bajo luz solar directa y salida RS485 Modbus RTU."
                    </p>
                    <DiagramWqmV100 />
                    <SpecSheet
                        title="EcoWest WQM-V100 (Sonda Óptica DO)"
                        subtitle="Datasheet de rendimiento óptico, eléctrico y de transductores limnológicos."
                        specs=wqm_specs
                    />
                </article>

                // Sistema 2: ESP32-P4 / C6 Gateway
                <article class="hardware-system-entry">
                    <h3 class="system-entry-title">"2. ESP32-P4 / C6 Edge Telemetry Gateway"</h3>
                    <p class="system-entry-desc">
                        "Concentrador de telemetría de arquitectura dual-chip. El núcleo de radio ESP32-C6 captura ráfagas ESP-NOW con Nanopb en microsegundos, transfiriendo los paquetes vía bus SPI al procesador host ESP32-P4 Dual-Core RISC-V para firma criptográfica ES256 y publicación asíncrona hacia GCP Pub/Sub."
                    </p>
                    <DiagramGateway />
                    <SpecSheet
                        title="Gateway Industrial Dual-Chip (ESP32-P4 / C6)"
                        subtitle="Especificaciones de procesamiento edge, criptografía de enlace y conectividad backhaul."
                        specs=gateway_specs
                    />
                </article>

                // Sistema 3: Smart Environmental & Soil Nodes
                <article class="hardware-system-entry">
                    <h3 class="system-entry-title">"3. Nodos Sensores de Monitoreo Ambiental y Edáfico"</h3>
                    <p class="system-entry-desc">
                        "Nodos autónomos de ultrabajo consumo para monitoreo de interiores y perfiles edáficos. Alojan transductores BME688 con BSEC compilada en firmware, sensor NDIR fotoacústico Sensirion SCD41 y sensor óptico de partículas BMV080 con auto-diagnóstico de lente obstruida."
                    </p>
                    <DiagramSensorNode />
                    <SpecSheet
                        title="Nodo Ambiental y Edáfico Multiparamétrico"
                        subtitle="Arreglo de sensores de estado sólido, tolerancias metrológicas y presupuesto de potencia."
                        specs=sensor_node_specs
                    />
                </article>
            </div>
        </div>
    }
}
