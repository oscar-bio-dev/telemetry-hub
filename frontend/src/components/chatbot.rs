// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.

use crate::utils::api_url;
use leptos::*;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ChatRequest {
    message: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    reply: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sender {
    User,
    Ai,
    System,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatMessage {
    pub id: usize,
    pub sender: Sender,
    pub text: String,
}

#[component]
pub fn Chatbot() -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);
    let (messages, set_messages) = create_signal(Vec::<ChatMessage>::new());
    let (input, set_input) = create_signal(String::new());
    let (is_loading, set_is_loading) = create_signal(false);
    let (cooldown_seconds, set_cooldown_seconds) = create_signal(0u32);
    let id_counter = create_rw_signal(0usize);

    let next_id = move || {
        let current = id_counter.get();
        id_counter.update(|c| *c += 1);
        current
    };

    let send_message = create_action(move |msg: &String| {
        let msg_text = msg.clone();
        async move {
            set_is_loading.set(true);
            let user_msg_id = next_id();
            set_messages.update(|m| {
                m.push(ChatMessage {
                    id: user_msg_id,
                    sender: Sender::User,
                    text: msg_text.clone(),
                });
            });

            let req_body = ChatRequest { message: msg_text };
            let target_url = api_url("/api/chat");
            let client = reqwest::Client::new();
            let res = client.post(&target_url).json(&req_body).send().await;

            match res {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        if let Ok(chat_resp) = resp.json::<ChatResponse>().await {
                            let ai_msg_id = next_id();
                            set_messages.update(|m| {
                                m.push(ChatMessage {
                                    id: ai_msg_id,
                                    sender: Sender::Ai,
                                    text: chat_resp.reply,
                                });
                            });
                        } else {
                            let sys_msg_id = next_id();
                            set_messages.update(|m| {
                                m.push(ChatMessage {
                                    id: sys_msg_id,
                                    sender: Sender::System,
                                    text: "Error parseando la respuesta del asistente.".to_string(),
                                });
                            });
                        }
                    } else if status == StatusCode::TOO_MANY_REQUESTS {
                        set_cooldown_seconds.set(12);
                        let interval = leptos::set_interval_with_handle(
                            move || {
                                set_cooldown_seconds.update(|sec| {
                                    if *sec > 0 {
                                        *sec -= 1;
                                    }
                                });
                            },
                            std::time::Duration::from_secs(1),
                        );
                        leptos::set_timeout(
                            move || {
                                if let Ok(h) = interval {
                                    h.clear();
                                }
                                set_cooldown_seconds.set(0);
                            },
                            std::time::Duration::from_secs(12),
                        );

                        let sys_msg_id = next_id();
                        set_messages.update(|m| {
                            m.push(ChatMessage {
                                id: sys_msg_id,
                                sender: Sender::System,
                                text: "Límite de tasa alcanzado (HTTP 429). Por favor espere 12 segundos antes de enviar otra consulta.".to_string(),
                            });
                        });
                    } else {
                        let err_text = resp.text().await.unwrap_or_default();
                        let sys_msg_id = next_id();
                        set_messages.update(|m| {
                            m.push(ChatMessage {
                                id: sys_msg_id,
                                sender: Sender::System,
                                text: if err_text.is_empty() {
                                    format!("API Error: {status}")
                                } else {
                                    format!("API Error {status}: {err_text}")
                                },
                            });
                        });
                    }
                }
                Err(e) => {
                    let sys_msg_id = next_id();
                    set_messages.update(|m| {
                        m.push(ChatMessage {
                            id: sys_msg_id,
                            sender: Sender::System,
                            text: format!("Error de red: {e}"),
                        });
                    });
                }
            }
            set_is_loading.set(false);
        }
    });

    let dispatch_send = move || {
        let trimmed = input.get().trim().to_string();
        if !trimmed.is_empty() && !is_loading.get() && cooldown_seconds.get() == 0 {
            set_input.set(String::new());
            send_message.dispatch(trimmed);
        }
    };

    let is_send_disabled = move || is_loading.get() || cooldown_seconds.get() > 0;
    let is_cooling_down = move || cooldown_seconds.get() > 0;

    let button_label = move || {
        let cool = cooldown_seconds.get();
        if cool > 0 {
            format!("WAIT ({cool}s)")
        } else if is_loading.get() {
            "SENDING...".to_string()
        } else {
            "SEND".to_string()
        }
    };

    view! {
        <aside class="chatbot-wrapper" aria-label="EcoTech Assistant">
            <div class="chatbot-window" class:open=move || is_open.get()>
                <div class="chatbot-header">
                    <span>"EcoTech Assistant"</span>
                </div>

                <div class="chatbot-messages" role="log" aria-live="polite" aria-relevant="additions">
                    <For
                        each=move || messages.get()
                        key=|m| m.id
                        children=move |m| {
                            let (msg_class, sender_label) = match m.sender {
                                Sender::User => ("chatbot-msg chatbot-msg-user", "User"),
                                Sender::System => ("chatbot-msg chatbot-msg-system", "System"),
                                Sender::Ai => ("chatbot-msg chatbot-msg-ai", "AI"),
                            };
                            view! {
                                <div class=msg_class>
                                    <span class="chatbot-sender">{sender_label}</span>
                                    <div class="chatbot-bubble">
                                        {m.text}
                                    </div>
                                </div>
                            }
                        }
                    />
                    {move || if is_loading.get() {
                        view! { <div class="chatbot-typing">"EcoTech is typing..."</div> }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </div>

                <div class="chatbot-input-bar">
                    <label for="chatbot-user-input" class="sr-only">
                        "Escribe tu mensaje para el asistente"
                    </label>
                    <input
                        id="chatbot-user-input"
                        type="text"
                        class="chatbot-input"
                        placeholder="Pregúntame sobre los sensores..."
                        prop:value=input
                        prop:disabled=is_cooling_down
                        on:input=move |ev| set_input.set(event_target_value(&ev))
                        on:keydown=move |ev| {
                            if ev.key() == "Enter" {
                                dispatch_send();
                            }
                        }
                    />
                    <button
                        type="button"
                        class="chatbot-btn-send"
                        prop:disabled=is_send_disabled
                        on:click=move |_| dispatch_send()
                    >
                        {button_label}
                    </button>
                </div>
            </div>

            <button
                type="button"
                class="chatbot-fab"
                aria-label="Abrir asistente de IA"
                on:click=move |_| set_is_open.update(|o| *o = !*o)
            ></button>
        </aside>
    }
}
