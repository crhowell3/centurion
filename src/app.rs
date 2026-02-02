use std::collections::HashMap;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use serde_json::Value;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[function_component(App)]
pub fn app() -> Html {
    let startup = Callback::from(|_| {
        spawn_local(async {
            match invoke("send_startup", JsValue::NULL).await {
                _ => web_sys::console::log_1(&"Sent startup command".into()),
            }
        })
    });

    let terminate = Callback::from(|_| {
        spawn_local(async {
            match invoke("send_terminate", JsValue::NULL).await {
                _ => web_sys::console::log_1(&"Sent termination command".into()),
            }
        })
    });

    let standby = Callback::from(|_| {
        spawn_local(async {
            match invoke("send_standby", JsValue::NULL).await {
                _ => web_sys::console::log_1(&"Sent standby command".into()),
            }
        })
    });

    let restart = Callback::from(|_| {
        spawn_local(async {
            match invoke("send_restart", JsValue::NULL).await {
                _ => web_sys::console::log_1(&"Sent restart command".into()),
            }
        })
    });

    html! {
        <body>
            <header>
                <strong>{"Centurion Dashboard"}</strong>
                <span style="margin-left:1rem; color:var{--muted}">
                    {"Exercise ID: "} <strong>{"001"}</strong>
                </span>
            </header>
            <main>
                <section class="panel">
                    <h2>{"Simulation Status"}</h2>
                    <div class="status-grid">
                        <div class="status-item">
                            <span class="label">{"State"}</span>
                            <span class="value">{"REPLACE"}</span>
                        </div>
                        <div class="status-item">
                            <span class="label">{"Sim Time"}</span>
                            <span class="value">{"00:00:00"}</span>
                        </div>
                        <div class="status-item">
                            <span class="label">{"Wall Time"}</span>
                            <span class="value">{"00:00:00"}</span>
                        </div>
                        <div class="status-item">
                            <span class="label">{"Active Entities"}</span>
                            <span class="value">{"0"}</span>
                        </div>
                    </div>
                </section>
                <section class="panel">
                    <h2>{"Global Controls"}</h2>
                    <div class="controls">
                        <button class="success" onclick={startup}>{"Start"}</button>
                        <button class="warning" onclick={standby}>{"Pause"}</button>
                        <button class="danger" onclick={terminate}>{"Stop"}</button>
                        <button onclick={restart}>{"Restart"}</button>
                    </div>
                </section>

                <section class="panel wide">
                    <h2>{"Alerts"}</h2>
                </section>
            </main>
        </body>
    }
}
