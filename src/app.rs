use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[function_component(App)]
pub fn app() -> Html {
    // let sim_state = app.state::<AppData>();
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
                    <h2>{"Network Health"}</h2>
                    <div class="status-grid">
                        <div class="status-item">
                            <span class="label">{"Incoming PDU Rate"}</span>
                            <span class="value">{"0 / s"}</span>
                        </div>
                        <div class="status-item">
                            <span class="label">{"Outgoing PDU Rate"}</span>
                            <span class="value">{"0 / s"}</span>
                        </div>
                        <div class="status-item">
                            <span class="label">{"Latency"}</span>
                            <span class="value">{"0 ms"}</span>
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
                        <li class="alert warning">{"Entity 3: Value may be out of range"}</li>
                    </ul>
                    <ul class="alerts">
                        <li class="alert error">{"Entity 3: Error processing header"}</li>
                    </ul>
                </section>
            </main>
            <footer>
                {"DIS Protocol: IEEE 1278.1-2012 | Site ID: 1 | Application ID: 10 | Status: Connected"}
            </footer>
        </body>
    }
}
