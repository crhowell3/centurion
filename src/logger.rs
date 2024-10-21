use std::{
    env, mem,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use chrono::Utc;
use log::Log;

pub use data::log::{Error, Record};
