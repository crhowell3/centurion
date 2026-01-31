// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(name = "centurion")]
#[command(
    version = centurion_lib::VERSION_AND_GIT_HASH,
    about = "A SIMAN application",
    author = "Cameron Howell <me@crhowell.com>",
    display_name = "centurion",
    help_template = "{name} {version}
author: {author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
"
)]
struct Args {
    /// Configuration file
    #[arg()]
    file: String,
}

fn main() {
    let args = Args::parse();
    let config_file = args.file;

    let _ = centurion_lib::run_cli(PathBuf::from(config_file));

    centurion_lib::run();
}
