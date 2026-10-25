// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.
// Proprietary and confidential.

use crate::domain::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::env;

/// Request DTO para el Chatbot.
#[derive(Deserialize, utoipa::ToSchema)]
pub struct ChatRequest {
    /// Consulta del usuario
    pub message: String,
}

/// Response DTO para el Chatbot.
#[derive(Serialize, utoipa::ToSchema)]
pub struct ChatResponse {
    /// Respuesta de la IA
    pub reply: String,
}

#[derive(Serialize)]
struct GeminiRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
    contents: Vec<GeminiContent>,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContentResp,
}

#[derive(Deserialize)]
struct GeminiContentResp {
    parts: Vec<GeminiPartResp>,
}

#[derive(Deserialize)]
struct GeminiPartResp {
    text: String,
}

/// Endpoint para consultar a la IA sobre el estado del Gemelo Digital.
#[utoipa::path(
    post,
    path = "/api/chat",
    request_body = ChatRequest,
    responses(
        (status = 200, description = "Respuesta de la IA", body = ChatResponse),
        (status = 500, description = "Error interno al comunicarse con Gemini")
    )
)]
#[allow(clippy::unused_async)]
pub async fn chat_with_twin(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 2. Extraer estado del Gemelo Digital
    let twin_data = {
        let state_map: std::collections::HashMap<String, shared::TelemetryPayload> =
            state.digital_twin.read().await.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        serde_json::to_string_pretty(&state_map).unwrap_or_default()
    };

    // 3. Construir el System Prompt para Gemini
    let system_prompt = format!(
        "Your name is EcoTech. You are an expert AI in conservation technology and environmental monitoring.\n\
        You specialize in analyzing telemetry data for ecological studies.\n\
        The current real-time telemetry from all active hardware nodes is provided below in JSON format:\n\
        {twin_data}\n\
        \n\
        Rules:\n\
        - Act as a conversational assistant. DO NOT analyze or list the telemetry data unless the user explicitly asks for it.\n\
        - If the user asks who you are or what you can do, introduce yourself and mention that you can analyze real-time biosensor telemetry.\n\
        - If they ask for telemetry data not present in the JSON, state clearly that the nodes are not providing that data right now.\n\
        - Be professional, but maintain a slightly cyberpunk/hacker tone suitable for an Agent Terminal UI."
    );

    let safe_message = payload.message.chars().take(2000).collect::<String>();

    // 4. Llamar a la API de Gemini
    let gemini_key = env::var("GEMINI_API_KEY").unwrap_or_default();

    if gemini_key.trim().is_empty() || gemini_key == "your_gemini_key_here" {
        tracing::warn!(
            "GEMINI_API_KEY no configurada o usando placeholder; respondiendo con simulación local"
        );
        let twin_summary = if twin_data == "{}" {
            "No active sensor telemetry detected in the Digital Twin RAM cache yet. (Start dummy publisher to stream data)".to_string()
        } else {
            format!("Digital Twin real-time telemetry snapshot:\n{twin_data}")
        };
        let fallback_reply = format!(
            "EcoTech Cognitive Agent [Local Simulation Mode]:\n\
            Telemetry reasoning engine is online.\n\n\
            {twin_summary}\n\n\
            (Nota: Configura una clave válida en GEMINI_API_KEY en tu archivo .env para activar el razonamiento generativo completo con Google Gemini)."
        );
        return Ok(Json(ChatResponse { reply: fallback_reply }));
    }

    let url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-flash-lite-latest:generateContent";

    let req_body = GeminiRequest {
        system_instruction: Some(GeminiContent { parts: vec![GeminiPart { text: system_prompt }] }),
        contents: vec![GeminiContent { parts: vec![GeminiPart { text: safe_message }] }],
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| {
            tracing::error!("Error building reqwest client: {}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Client error".to_string())
        })?;

    tracing::info!("Enviando request a Gemini API...");
    let res = client
        .post(url)
        .header("x-goog-api-key", gemini_key)
        .json(&req_body)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Fallo en request a Gemini: {}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "AI request failed".to_string())
        })?;
    tracing::info!("Gemini API respondió con status: {}", res.status());

    if !res.status().is_success() {
        let err_status = res.status();
        let err_body = res.text().await.unwrap_or_default();
        tracing::error!("Gemini API error {}: {}", err_status, err_body);
        return Err((StatusCode::BAD_GATEWAY, format!("Gemini API error ({err_status})")));
    }

    let gemini_resp: GeminiResponse = res.json().await.map_err(|e| {
        tracing::error!("Error parseando Gemini response: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "AI Response parse error".to_string())
    })?;

    // Extraer el texto de la respuesta
    let reply_text = gemini_resp
        .candidates
        .and_then(|mut c| c.pop())
        .and_then(|mut c| c.content.parts.pop())
        .map_or_else(|| "No response generated by the AI.".to_string(), |p| p.text);

    Ok(Json(ChatResponse { reply: reply_text }))
}
