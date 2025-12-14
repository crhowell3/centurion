use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

#[function_component(App)]
pub fn app() -> Html {
    let greet_input_ref = use_node_ref();

    let name = use_state(String::new);

    let greet_msg = use_state(String::new);
    {
        let greet_msg = greet_msg;
        let name = name;
        let name2 = name.clone();
        use_effect_with(name2, move |_| {
            spawn_local(async move {
                if name.is_empty() {
                    return;
                }

                let args = serde_wasm_bindgen::to_value(&GreetArgs { name: &name })
                    .expect("Should convert to JsValue");
                // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
                let new_msg = invoke("greet", args)
                    .await
                    .as_string()
                    .expect("Should convert to string");
                greet_msg.set(new_msg);
            });

            || {}
        });
    }

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
                            <span class="value">{"Running"}</span>
                        </div>
                        <div class="status-item">
                            <span class="label">{"Sim Time"}</span>
                            <span class="value">{"00:12:34"}</span>
                        </div>
                        <div class="status-item">
                            <span class="label">{"Wall Time"}</span>
                            <span class="value">{"00:12:40"}</span>
                        </div>
                        <div class="status-item">
                            <span class="label">{"Active Entities"}</span>
                            <span class="value">{"128"}</span>
                        </div>
                    </div>
                </section>
                <section class="panel">
                    <h2>{"Network Health"}</h2>
                    <div class="status-grid">
                        <div class="status-item">
                            <span class="label">{"PDU Rate"}</span>
                            <span class="value">{"100 / s"}</span>
                        </div>
                        <div class="status-item">
                            <span class="label">{"Latency"}</span>
                            <span class="value">{"18 ms"}</span>
                        </div>
                        <div class="status-item">
                            <span class="label">{"Multicast Group"}</span>
                            <span class="value">{"239.1.1.1"}</span>
                        </div>
                    </div>
                </section>
                <section class="panel">
                    <h2>{"Global Controls"}</h2>
                    <div class="controls">
                        <button class="success">{"Start"}</button>
                        <button class="warning">{"Pause"}</button>
                        <button class="danger">{"Stop"}</button>
                        <button>{"Reset"}</button>
                    </div>
                </section>

                <section class="panel wide">
                    <h2>{"Alerts"}</h2>
                    <ul class="alerts">
                        <li class="alert warning">{"Entity 3: Error"}</li>
                    </ul>
                </section>
            </main>
            <footer>
                {"DIS Protocol: IEEE 1278.1-2012 | Site ID: 1 | Application ID: 10 | Status: Connected"}
            </footer>
        </body>
    }
}
