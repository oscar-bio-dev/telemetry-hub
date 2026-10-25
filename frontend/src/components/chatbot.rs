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
    let (jwt_token, set_jwt_token) = create_signal(None::<String>);

    // Fetch initial JWT token on mount
    create_effect(move |_| {
        spawn_local(async move {
            let window = web_sys::window().expect("no global `window` exists");
            let location = window.location();
            let host = location.host().expect("should have a host");
            let protocol = location.protocol().expect("should have a protocol");
            let auth_url = if host.contains("localhost") || host.contains("127.0.0.1") {
                "http://127.0.0.1:3000/api/auth/mock-login".to_string()
            } else {
                format!("{protocol}//{host}/api/auth/mock-login")
            };

            if let Ok(res) = reqwest::Client::new().post(&auth_url).send().await {
                if let Ok(json) = res.json::<serde_json::Value>().await {
                    if let Some(token) = json.get("token").and_then(|t| t.as_str()) {
                        set_jwt_token.set(Some(token.to_string()));
                    }
                }
            }
        });
    });

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

            // Ensure JWT token is available
            let token = if let Some(t) = jwt_token.get_untracked() {
                Some(t)
            } else {
                let auth_url = if host.contains("localhost") || host.contains("127.0.0.1") {
                    "http://127.0.0.1:3000/api/auth/mock-login".to_string()
                } else {
                    format!("{protocol}//{host}/api/auth/mock-login")
                };
                if let Ok(res) = reqwest::Client::new().post(&auth_url).send().await {
                    if let Ok(json) = res.json::<serde_json::Value>().await {
                        let t =
                            json.get("token").and_then(|tok| tok.as_str()).map(ToString::to_string);
                        if let Some(ref tok_str) = t {
                            set_jwt_token.set(Some(tok_str.clone()));
                        }
                        t
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            let client = reqwest::Client::new();
            let mut req = client.post(&api_url).json(&req_body);
            if let Some(token_val) = token {
                req = req.header("Authorization", format!("Bearer {token_val}"));
            }
            let res = req.send().await;

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
