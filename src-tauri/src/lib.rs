use std::fmt;

use tauri::{Builder, Manager};

#[derive(Debug)]
enum SimulationState {
    Stopped,
    Standby,
    Running,
}

impl fmt::Display for SimulationState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

struct AppData {
    simulation_state: &'static SimulationState,
}

#[tauri::command]
const fn send_startup() {
    // NOOP
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
///
/// # Panics
/// - May panic if tauri fails to generate context
///
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![send_startup])
        .setup(|app| {
            app.manage(AppData {
                simulation_state: &SimulationState::Stopped,
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
