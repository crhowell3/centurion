use std::fmt;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq)]
enum NotificationLevel {
    Info,
    Warning,
    Error,
}

impl fmt::Display for NotificationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct Notification {
    message: String,
    level: NotificationLevel,
}

#[derive(Clone, PartialEq, Eq)]
struct Notifications(Vec<Notification>);

enum NotificationAction {
    Push(Notification),
}

impl Reducible for Notifications {
    type Action = NotificationAction;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        match action {
            NotificationAction::Push(n) => {
                let mut list = self.0.clone();
                list.push(n);
                Self(list).into()
            }
        }
    }
}

#[function_component(Dashboard)]
pub fn dashboard() -> Html {
    let notifications = use_reducer(|| Notifications(Vec::new()));

    let append_notification = {
        let notifications = notifications.clone();

        Callback::from(move |notification: Notification| {
            notifications.dispatch(NotificationAction::Push(notification));
        })
    };

    let send_siman_pdu = {
        let notify = append_notification.clone();

        Callback::from(move |command: &'static str| {
            let notify = notify.clone();

            spawn_local(async move {
                let payload = serde_json::json!({"command": command});

                let result = invoke(
                    "send_siman_pdu",
                    serde_wasm_bindgen::to_value(&payload).unwrap_or_default(),
                )
                .await;
                match result {
                    Ok(_) => {
                        // noop
                    }
                    Err(err) => {
                        notify.emit(Notification {
                            message: err.as_string().unwrap_or_else(|| "unknown error".into()),
                            level: NotificationLevel::Error,
                        });
                    }
                }
            });
        })
    };

    let send_command = |cmd: &'static str| {
        let send = send_siman_pdu.clone();
        let notify = append_notification.clone();

        Callback::from(move |_| {
            send.emit(cmd);
            notify.emit(Notification {
                message: format!("{} command sent", cmd.to_uppercase()),
                level: NotificationLevel::Info,
            });
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
                        {notifications.0.iter().rev().map(|n| {
                            let class = match n.level {
                                NotificationLevel::Info => "info",
                                NotificationLevel::Warning => "warning",
                                NotificationLevel::Error => "error",
                            };

                            html! {
                                <li class={format!("alert {class}")}>{format!("[{}] {}", &n.level, &n.message)}</li>
                            }
                        }).collect::<Html>()
                        }
                    </ul>
                </section>
            </main>
        </body>
    }
}
