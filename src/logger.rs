use std::{
    env, mem,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use chrono::Utc;
use log::Log;

pub use data::log::{Error, Record};

pub fn setup(is_debug: bool) -> Result<(), Error> {
    let level_filter = env::var("RUST_LOG")
        .ok()
        .as_deref()
        .map(str::parse::<log::Level>)
        .transpose()?
        .unwrap_or(log::Level::Debug)
        .to_level_filter();

    let mut io_sink = fern::Dispatch::new().format(|out, message, record| {
        out.finish(format_args!("[{}] {}", record.level(), message))
    });

    if is_debug {
        io_sink = io_sink.chain(std::io::stdout());
    } else {
        let log_file = data::log::file()?;

        io_sink = io_sink.chain(log_file);
    }

    fern::Dispatch::new()
        .level(log::LevelFilter::Off)
        .level_for("panic", log::LevelFilter::Error)
        .level_for("iced_wgpu", log::LevelFilter::Info)
        .level_for("data", level_filter)
        .level_for("centurion", level_filter)
        .chain(io_sink)
        .apply()?;

    Ok(())
}
