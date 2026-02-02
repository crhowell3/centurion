use std::fmt;

use std::sync::Mutex;

#[derive(Debug)]
pub enum SimulationState {
    Stopped,
    Standby,
    Running,
}

#[derive(Debug)]
pub struct RequestIds {
    pub start_resume: u32,
    pub stop_freeze: u32,
    pub action_request: u32,
}

impl RequestIds {
    pub fn new() -> Self {
        RequestIds {
            start_resume: 0,
            stop_freeze: 0,
            action_request: 0,
        }
    }
}

impl fmt::Display for SimulationState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

pub struct AppState {
    pub simulation_state: Mutex<SimulationState>,
    pub request_ids: Mutex<RequestIds>,
}
