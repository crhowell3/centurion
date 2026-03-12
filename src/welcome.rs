use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Properties, PartialEq)]
pub struct WelcomeModalProps {
    pub on_loaded: Callback<crate::Config>,
}

#[function_component(WelcomeModal)]
pub fn welcome_modal(props: &WelcomeModalProps) -> Html {
    let error_message = use_state(|| None::<String>);

    let on_click = {
        let on_loaded = props.on_loaded.clone();
        let error_message = error_message.clone();

        Callback::from(move |_| {
            let on_loaded = on_loaded.clone();
            let error_message = error_message.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let result = invoke("load_scenario_config", JsValue::NULL).await;

                match result {
                    Ok(_) => {
                        let dummy_config = crate::Config {
                            multicast_address: String::new(),
                            port: 1,
                        };
                        on_loaded.emit(dummy_config);
                    }
                    Err(err) => {
                        error_message
                            .set(Some(err.as_string().unwrap_or_else(|| {
                                "unknown error received from backend".into()
                            })));
                    }
                }
            });
        })
    };

    let close_error = {
        let error_message = error_message.clone();
        Callback::from(move |_| error_message.set(None))
    };

    html! {
        <>
        <div class="modal-backdrop">
            <div class="modal">
                <img src="assets/Square150x150Logo.png" />
                <h1>{ "Welcome to Centurion" }</h1>
                <p>{ "Select a configuration file to get started." }</p>
                <button onclick={on_click}>{ "LOAD" }</button>
            </div>
        </div>

        {
            error_message.as_ref().map_or_else(|| html! {}, |err| html! {
                <div class="modal-backdrop">
                    <div class="modal error">
                        <h2>{ "Error Loading Config" }</h2>
                        <p>{ err }</p>
                        <button onclick={close_error}>{ "OK" }</button>
                    </div>
                </div>
            })
        }
        </>
    }
}
