// Hide terminal on windows release builds
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]
use clap::{command, arg};
use inkbound_parser::parse_log_to_json;

use std::time::Duration;

use logreader::LogReader;

#[inline(always)]
fn default_logpath() -> String {
    // Use a local log file for test purposes
    // TODO: consider committing some trimmed down, sanitized logs to the repo for test cases
    #[cfg(debug_assertions)]
    return "./logfile.log".to_string();

    #[cfg(not(debug_assertions))]
    {
        #[cfg(target_os = "windows")]
        return format!("{}\\AppData\\LocalLow\\Shiny Shoe\\Inkbound\\logfile.log", std::env::var("USERPROFILE").unwrap());

        #[cfg(target_os = "linux")]
        return format!("{}/.steam/steam/steamapps/compatdata/1062810/pfx/drive_c/users/steamuser/AppData/LocalLow/Shiny Shoe/Inkbound/logfile.log", std::env::var("HOME").unwrap());
    }
}

fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    // env_logger::Builder::from_env(Env)
    //     .filter_level(log::LevelFilter::Debug)
    //     .init();

    let matches = command!()
        // TODO: Consider making this a subcommand, so that required args can be set properly
        .arg(arg!(-p --parse <FILE> "Parse a single log into a json string")
            .required(false)
        )
        .arg(arg!(-f --file <FILE> "File to parse and watch for updates")
            .required(false)
        )
        .arg(arg!(-s --"skip-current" "Skip over parsing current log file")
            .required(false)
            .action(clap::ArgAction::SetTrue)
        )
        .arg(arg!(-w --windowed "Render in a window instead of as a borderless fullscreen overlay")
            .required(false)
            .action(clap::ArgAction::SetTrue)
        )
    ;

    #[cfg(feature = "auto_update")]
    let matches = matches
        .arg(arg!(--update "Run self-updater and exit")
            .required(false)
            .action(clap::ArgAction::SetTrue)
        );

    let matches = matches
        .get_matches();

    // Initialize Updater
    #[cfg(feature = "auto_update")]
    {
        let mut updater = updater::UPDATER.lock().unwrap();
        updater.init(env!("CARGO_PKG_VERSION").to_string());

        if *matches.get_one("update").unwrap_or(&false) {
            log::debug!("current version = {}", updater.current_version);
            updater.set_options(updater::UpdaterOptions::default().show_download_progress(true));

            updater.do_update(true);

            if let updater::UpdateStatus::Error(e) = &*updater.status.lock().unwrap() {
                log::error!("Error updating: {e}");
            }

            return
        }
    }

    // Parse-only mode
    if let Some(file) = matches.get_one::<String>("parse") {
        println!("{}", parse_log_to_json(file));
        return
    }

    let filepath = if let Some(filepath) = matches.get_one::<String>("file") {
        filepath.to_owned()
    } else {
        default_logpath()
    };

    let skip_current = matches.get_flag("skip-current");
    let reader = LogReader::new(filepath.clone(), Duration::from_secs(2), skip_current);

    log::info!("starting watch of file: {}", filepath);

    #[cfg(not(debug_assertions))]
    let mode = if matches.get_flag("windowed") {
        overlay::OverlayMode::WindowedOverlay
    } else {
        overlay::OverlayMode::Overlay
    };
    // Just always use windowed mode in debug builds
    #[cfg(debug_assertions)]
    let mode = overlay::OverlayMode::WindowedOverlay;
    overlay::spawn_overlay(reader, mode);
}
