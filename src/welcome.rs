use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Properties, PartialEq)]
pub struct WelcomeModalProps {
    pub on_loaded: Callback<crate::Config>,
}

#[function_component(WelcomeModal)]
pub fn welcome_modal(props: &WelcomeModalProps) -> Html {
    let on_click = {
        let on_loaded = props.on_loaded.clone();
        Callback::from(move |_| {
            let on_loaded = on_loaded.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let _ = invoke("load_scenario_config", JsValue::NULL).await;
            });
        })
    };

    html! {
        <div class="modal-backdrop">
            <div class="modal">
                <h1>{ "Welcome to Centurion" }</h1>
                <p>{ "Select a configuration file to get started." }</p>
                <button onclick={on_click}>{ "LOAD" }</button>
            </div>
        </div>
    }
}
