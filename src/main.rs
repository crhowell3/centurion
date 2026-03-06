mod app;
mod dashboard;
mod welcome;

use app::App;

use serde::Deserialize;

#[derive(Clone, PartialEq, Eq)]
pub enum AppStatus {
    WaitingForConfig,
    Loaded(Config),
}

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct Config {
    pub multicast_address: String,
    pub port: u16,
}

fn main() {
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
