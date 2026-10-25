// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.

use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ChatRequest {
    message: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    reply: String,
}

#[derive(Clone, Debug)]
struct ChatMessage {
    sender: String,
    text: String,
}

#[component]
pub fn Chatbot() -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);
    let (messages, set_messages) = create_signal(Vec::<ChatMessage>::new());
    let (input, set_input) = create_signal(String::new());
    let (is_loading, set_is_loading) = create_signal(false);

    let send_message = create_action(move |msg: &String| {
        let msg_clone = msg.clone();
        async move {
            set_is_loading.set(true);
            set_messages.update(|m| {
                m.push(ChatMessage { sender: "User".to_string(), text: msg_clone.clone() });
            });

            let req_body = ChatRequest { message: msg_clone };

            let window = web_sys::window().expect("no global `window` exists");
            let location = window.location();
            let host = location.host().expect("should have a host");
            let protocol = location.protocol().expect("should have a protocol");
            let api_url = if host.contains("localhost") || host.contains("127.0.0.1") {
                "http://127.0.0.1:3000/api/chat".to_string()
            } else {
                format!("{protocol}//{host}/api/chat")
            };

            let client = reqwest::Client::new();
            let res = client.post(&api_url).json(&req_body).send().await;

            match res {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        if let Ok(chat_resp) = resp.json::<ChatResponse>().await {
                            set_messages.update(|m| {
                                m.push(ChatMessage {
                                    sender: "AI".to_string(),
                                    text: chat_resp.reply,
                                });
                            });
                        } else {
                            set_messages.update(|m| {
                                m.push(ChatMessage {
                                    sender: "System".to_string(),
                                    text: "Error parsing AI response.".to_string(),
                                });
                            });
                        }
                    } else {
                        let err_text = resp.text().await.unwrap_or_default();
                        set_messages.update(|m| {
                            m.push(ChatMessage {
                                sender: "System".to_string(),
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
                    set_messages.update(|m| {
                        m.push(ChatMessage {
                            sender: "System".to_string(),
                            text: format!("Network Error: {e}"),
                        });
                    });
                }
            }
            set_is_loading.set(false);
            set_input.set(String::new());
        }
    });

    view! {
        <aside class="chatbot-wrapper" aria-label="EcoTech Assistant">
            <div class="chatbot-window" class:open=move || is_open.get()>
                <div class="chatbot-header">
                    <span>"EcoTech Assistant"</span>
                </div>

                <div class="chatbot-messages">
                    {move || messages.get().into_iter().map(|m| {
                        let is_user = m.sender == "User";
                        let is_system = m.sender == "System";
                        let msg_class = if is_user {
                            "chatbot-msg chatbot-msg-user"
                        } else if is_system {
                            "chatbot-msg chatbot-msg-system"
                        } else {
                            "chatbot-msg chatbot-msg-ai"
                        };
                        view! {
                            <div class=msg_class>
                                <span class="chatbot-sender">{m.sender}</span>
                                <div class="chatbot-bubble">
                                    {m.text}
                                </div>
                            </div>
                        }
                    }).collect_view()}
                    {move || if is_loading.get() {
                        view! { <div class="chatbot-typing">"EcoTech is typing..."</div> }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </div>

                <div class="chatbot-input-bar">
                    <input
                        type="text"
                        class="chatbot-input"
                        placeholder="Pregúntame sobre los sensores..."
                        prop:value=input
                        on:input=move |ev| set_input.set(event_target_value(&ev))
                        on:keydown=move |ev| {
                            if ev.key() == "Enter" && !input.get().trim().is_empty() {
                                send_message.dispatch(input.get());
                            }
                        }
                    />
                    <button
                        type="button"
                        class="chatbot-btn-send"
                        on:click=move |_| {
                            if !input.get().trim().is_empty() {
                                send_message.dispatch(input.get());
                            }
                        }
                    >
                        "SEND"
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
