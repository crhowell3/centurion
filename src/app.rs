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

fn command_button(send: Callback<&'static str>, command: &'static str) -> Callback<MouseEvent> {
    Callback::from(move |_| send.emit(command))
}

#[function_component(App)]
pub fn app() -> Html {
    let send_siman_pdu = Callback::from(|command: &'static str| {
        spawn_local(async move {
            let payload = serde_json::json!({"command": command});
            let _ = invoke(
                "send_siman_pdu",
                serde_wasm_bindgen::to_value(&payload).unwrap(),
            )
            .await;
            web_sys::console::log_1(&format!("Sent {} command", command).into());
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
                        <button class="success" onclick={command_button(send_siman_pdu.clone(), "startup")}>{"Start"}</button>
                        <button class="warning" onclick={command_button(send_siman_pdu.clone(), "standby")}>{"Pause"}</button>
                        <button class="danger" onclick={command_button(send_siman_pdu.clone(), "terminate")}>{"Stop"}</button>
                        <button onclick={command_button(send_siman_pdu.clone(), "reset")}>{"Restart"}</button>
                    </div>
                </section>

                <section class="panel wide">
                    <h2>{"Alerts"}</h2>
                </section>
            </main>
        </body>
    }
}
