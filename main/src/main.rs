#![windows_subsystem = "windows"] // Hide terminal on windows
use clap::{command, arg};
use inkbound_parser::{
    parser::{*},
    parse_log_to_json,
};

use std::{sync::{Arc, RwLock}, io::BufReader, fs::File, time::Duration, path::Path};
use std::io::{
    Seek,
    BufRead
};

use notify::{Watcher, RecursiveMode};

struct LogReader {
    reader: BufReader<File>,
    cache_string: String,
    cache_events: Vec<inkbound_parser::parser::Event>,
}

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

impl LogReader {
    fn new(filepath: &str) -> Self {
        let file = File::open(filepath).unwrap(); // TODO: unwrap

        let reader = std::io::BufReader::new(file);
        Self {
            reader,
            cache_string: String::new(),
            cache_events: Vec::new(),
        }
    }

    // TODO: consider having LogReader contain the LogParser. LogParser may not need to be shared
    fn reader_to_datalog(&mut self, parser: &Arc<RwLock<LogParser>>, datalog: &Arc<RwLock<DataLog>>) {
        let mut parser = parser.write().unwrap();

        while self.reader.read_line(&mut self.cache_string).is_ok_and(|size| size != 0) {
            log::trace!("parsing: {}", self.cache_string);
            if let Some(event) = parser.parse_line(&self.cache_string.as_str()) {
                self.cache_events.push(event);
                // datalog.handle_event(event);
            }
            self.cache_string.clear();
        }

        // Events collected, now acquire write lock to update the datalog
        {
            let mut datalog = datalog.write().unwrap();
            // TODO: consider changing .handle_events to accept an iterator
            for event in self.cache_events.drain(..) {
                datalog.handle_event(event);
            }
        }
    }

    fn seek_end(&mut self) {
        self.reader.seek(std::io::SeekFrom::End(0)).unwrap();
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

    let parser = Arc::new(RwLock::new(LogParser::new()));
    let datalog = Arc::new(RwLock::new(DataLog::new()));

    let filepath = if let Some(filepath) = matches.get_one::<String>("file") {
        filepath.to_owned()
    } else {
        default_logpath()
    };
    let mut reader = LogReader::new(filepath.as_str());

    // Catch the parser/log up...
    let (backlog_tx, backlog_rx) = std::sync::mpsc::channel();
    {
        let datalog = datalog.clone();
        let parser = parser.clone();
        let skip = matches.get_flag("skip-current");

        std::thread::spawn(move || {
            if skip {
                log::debug!("skipping backlog");
                reader.seek_end();
            } else {
                log::debug!("reading from backlog");
                reader.reader_to_datalog(&parser, &datalog);
            }
            backlog_tx.send(reader).unwrap();
        });
    }

    let (tx, rx) = std::sync::mpsc::channel();

    // ...then start the watcher
    let watch_handle = {
        let parser = parser.clone();
        let datalog = datalog.clone();
        std::thread::spawn(move || {
            log::debug!("waiting to receive reader from backlog thread...");
            let mut reader = backlog_rx.recv().unwrap();
            drop(backlog_rx);
            log::debug!("received reader");

            log::debug!("spawning watcher receive thread");
            loop {
                match rx.recv() {
                    Ok(Ok(_events)) => {
                        log::trace!("file update received");
                        reader.reader_to_datalog(&parser, &datalog);
                    },
                    Ok(Err(e)) => {
                        log::error!("Error from watcher: {:?}, closing thread", e);
                        break;
                    },
                    Err(_) => {
                        log::debug!("channel closed, exiting watch recv loop");
                        break;
                    },
                };
            }
            log::debug!("closing watch recv thread");
        })
    };

    log::info!("starting watch of file: {}", filepath);

    // TODO: allow configuration of poll duration
    let mut watcher = notify::PollWatcher::new(tx, notify::Config::default().with_poll_interval(Duration::from_secs(2))).unwrap();
    watcher.watch(Path::new(filepath.as_str()), RecursiveMode::NonRecursive).unwrap();

    #[cfg(not(debug_assertions))]
    let mode = if matches.get_flag("windowed"){
        overlay::OverlayMode::WindowedOverlay
    } else {
        overlay::OverlayMode::Overlay
    };
    // Just always use windowed mode in debug builds
    #[cfg(debug_assertions)]
    let mode = overlay::OverlayMode::WindowedOverlay;
    overlay::spawn_overlay(datalog, mode);

    // Overlay closed, exit and clean-up
    drop(watcher); // Drop to close the watch recv thread loop
    watch_handle.join().unwrap();
}
