use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[function_component(Dashboard)]
pub fn dashboard() -> Html {
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

    let notifications = use_state(|| Vec::<String>::new());

    let notifications_handle = notifications.clone();

    let append_notification = Callback::from(move |message: String| {
        let mut new_notifications = (*notifications_handle).clone();
        new_notifications.push(message);
        notifications_handle.set(new_notifications);
    });

    let send_command = |cmd: &'static str| {
        let send = send_siman_pdu.clone();
        let notify = append_notification.clone();

        Callback::from(move |_| {
            send.emit(cmd);
            notify.emit(format!("{} command sent", cmd.to_uppercase()));
        })
    };

    html! {
        <body>
            <header>
                <strong>{"Centurion Dashboard"}</strong>
                <span style="margin-left:1rem; color:var{--muted}">
                    {"Exercise ID: "} <strong>{"001"}</strong>
                </span>
            </header>
            <main>
                <section class="panel wide">
                    <h2>{"Global Controls"}</h2>
                    <div class="controls">
                        <button class="primary" onclick={send_command("initialize")}>{"Initialize"}</button>
                        <button class="success" onclick={send_command("startup")}>{"Operate"}</button>
                        <button class="warning" onclick={send_command("standby")}>{"Pause"}</button>
                        <button class="danger" onclick={send_command("terminate")}>{"Shutdown"}</button>
                        <button onclick={send_command("reset")}>{"Restart"}</button>
                    </div>
                </section>

                <section class="panel wide">
                    <h2>{"Notifications"}</h2>
                    <ul class="alerts">
                        {notifications.iter().rev().map(|message| html! {
                            <li class="alert info">{format!("[INFO] {message}")}</li>
                        }).collect::<Html>()}
                    </ul>
                </section>
            </main>
        </body>
    }
}
