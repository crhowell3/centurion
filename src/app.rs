use wasm_bindgen::prelude::*;
use yew::prelude::*;

use crate::dashboard::Dashboard;
use crate::welcome::WelcomeModal;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[function_component(App)]
pub fn app() -> Html {
    let status = use_state(|| crate::AppStatus::WaitingForConfig);

    let on_config_loaded = {
        let status = status.clone();

        Callback::from(move |config: crate::Config| {
            status.set(crate::AppStatus::Loaded(config));
        })
    };

    html! {
        <>
        {
            match &*status {
                crate::AppStatus::WaitingForConfig => html! {
                    <WelcomeModal on_loaded={on_config_loaded}/>
                },
                crate::AppStatus::Loaded(_config) => html! {
                    <Dashboard />
                },
            }
        }
        </>
    }
}
