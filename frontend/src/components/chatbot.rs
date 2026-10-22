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
                m.push(ChatMessage { sender: "User".to_string(), text: msg_clone.clone() })
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
                    if resp.status().is_success() {
                        if let Ok(chat_resp) = resp.json::<ChatResponse>().await {
                            set_messages.update(|m| {
                                m.push(ChatMessage {
                                    sender: "AI".to_string(),
                                    text: chat_resp.reply,
                                })
                            });
                        } else {
                            set_messages.update(|m| {
                                m.push(ChatMessage {
                                    sender: "System".to_string(),
                                    text: "Error parsing AI response.".to_string(),
                                })
                            });
                        }
                    } else {
                        set_messages.update(|m| {
                            m.push(ChatMessage {
                                sender: "System".to_string(),
                                text: format!("API Error: {}", resp.status()),
                            })
                        });
                    }
                }
                Err(e) => {
                    set_messages.update(|m| {
                        m.push(ChatMessage {
                            sender: "System".to_string(),
                            text: format!("Network Error: {e}"),
                        })
                    });
                }
            }
            set_is_loading.set(false);
            set_input.set(String::new());
        }
    });

    view! {
        <div style="position: fixed; bottom: 20px; right: 20px; z-index: 9999;">
            <div style=move || format!("display: {}; flex-direction: column; width: 350px; height: 500px; background: var(--bg-hard); border: 2px solid var(--accent-yellow); border-radius: 8px; margin-bottom: 10px; overflow: hidden; box-shadow: 0 4px 15px rgba(0,0,0,0.5);", if is_open.get() { "flex" } else { "none" })>
                <div style="background: var(--accent-yellow); color: var(--bg-hard); padding: 10px; font-weight: bold; text-align: center;">
                    "EcoTech Assistant"
                </div>

                <div style="flex-grow: 1; padding: 10px; overflow-y: auto; display: flex; flex-direction: column; gap: 8px; background: var(--bg-soft);">
                    {move || messages.get().into_iter().map(|m| {
                        let is_user = m.sender == "User";
                        let color = if is_user { "var(--accent-blue)" } else { "var(--accent-yellow)" };
                        let align = if is_user { "flex-end" } else { "flex-start" };
                        view! {
                            <div style=format!("align-self: {}; max-width: 80%;", align)>
                                <div style=format!("font-size: 0.8em; color: {}; margin-bottom: 2px;", color)>{m.sender}</div>
                                <div style=format!("background: var(--border-color); padding: 8px; border-radius: 4px; border-left: 3px solid {}; white-space: pre-wrap;", color)>
                                    {m.text}
                                </div>
                            </div>
                        }
                    }).collect_view()}
                    {move || if is_loading.get() {
                        view! { <div style="color: var(--accent-blue); font-style: italic;">"EcoTech is typing..."</div> }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </div>

                <div style="padding: 10px; border-top: 1px solid var(--border-color); background: var(--bg-hard); display: flex;">
                    <input
                        type="text"
                        style="flex-grow: 1; background: var(--bg-soft); color: var(--fg-main); border: 1px solid var(--border-color); padding: 8px; outline: none;"
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
                        style="background: var(--accent-yellow); color: var(--bg-hard); border: none; padding: 0 15px; font-weight: bold; cursor: pointer;"
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
                on:click=move |_| set_is_open.update(|o| *o = !*o)
                style="width: 60px; height: 60px; border-radius: 50%; border: 2px solid var(--accent-yellow); background-color: var(--bg-soft); background-image: url('/public/assets/img/ia-avatar.png'); background-size: cover; background-position: center; cursor: pointer; float: right; box-shadow: 0 4px 10px rgba(0,0,0,0.5); transition: transform 0.2s;"
                onmouseover="this.style.transform='scale(1.1)'"
                onmouseout="this.style.transform='scale(1)'"
            ></button>
        </div>
    }
}
