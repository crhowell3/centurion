// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::path::PathBuf;

fn main() {
    let mut args = env::args().skip(1);

    if let Some(config_path) = args.next() {
        centurion_lib::run_cli(PathBuf::from(config_path));
    }
    centurion_lib::run();
}
