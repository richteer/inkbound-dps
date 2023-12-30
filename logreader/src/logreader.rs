use std::{
    sync::{
        Arc,
        RwLock,
        mpsc::{Sender, Receiver},
        atomic::Ordering
    },
    fs::File,
    time::Duration,
    path::Path,
    thread::JoinHandle
};
use std::io::{
    Seek,
    BufRead
};

use notify::{Watcher, RecursiveMode};

use inkbound_parser::parser::*;
use atomic_enum::atomic_enum;

pub enum LogReaderCommand {
    Update,
    Stop,
}

#[atomic_enum]
#[derive(PartialEq)]
pub enum LogReaderStatus {
    Initializing = 1,
    Reading,
    Idle,
    Errored,
}

impl std::fmt::Display for LogReaderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            LogReaderStatus::Initializing => "Initializing",
            LogReaderStatus::Reading => "Reading",
            LogReaderStatus::Idle => "Idle",
            LogReaderStatus::Errored => "Errored",
        })
    }
}

pub struct LogReader {
    // Configuration items
    filepath: String,
    poll_duration: Duration,
    state: LogReaderState,
}

// All the non-configuration items that need to be (re)initialized
struct LogReaderState {
    datalog: Arc<RwLock<DataLog>>,
    sender: Sender<LogReaderCommand>,
    status: Arc<AtomicLogReaderStatus>,
    datalog_thread: Option<JoinHandle<()>>,
    _watcher: Box<dyn Watcher>,
}

fn start_watcher(sender: Sender<LogReaderCommand>, status: Arc<AtomicLogReaderStatus>, filepath: &str, poll_duration: Duration) -> Box<dyn Watcher> {
    let watcher_callback =
        move |event| {
            match event {
                Ok(_events) => {
                    log::trace!("file update received");
                    if let Err(e) = sender.send(LogReaderCommand::Update) {
                        log::error!("Error sending update to logging thread: {e:?}");
                    }
                },
                Err(e) => {
                    log::error!("Error from watcher: {:?}", e);
                    status.store(LogReaderStatus::Errored, Ordering::Relaxed);
                },
            }
        };

    let mut watcher = notify::PollWatcher::new(watcher_callback, notify::Config::default().with_poll_interval(poll_duration)).unwrap();
    watcher.watch(Path::new(filepath), RecursiveMode::NonRecursive).unwrap();
    Box::new(watcher)
}

fn check_exit(rx: &Receiver<LogReaderCommand>) -> bool {
    match rx.try_recv() {
        Ok(LogReaderCommand::Update) => false,
        Err(std::sync::mpsc::TryRecvError::Empty) => false,

        Ok(LogReaderCommand::Stop) => {
            log::debug!("stop command received, exiting thread mid read");
            true
        },
        Err(e) => {
            log::debug!("unable to check for stop, exiting thread: {e:?}");
            true
        },
    }
}

fn init_datalog_thread(filepath: &str, status: Arc<AtomicLogReaderStatus>, sender: Sender<LogReaderCommand>, rx: Receiver<LogReaderCommand>, datalog: Arc<RwLock<DataLog>>, skip_current: bool) -> JoinHandle<()> {
    let file = File::open(filepath).unwrap(); // TODO: unwrap
    let mut reader = std::io::BufReader::new(file);

    let mut parser = LogParser::new();
    let datalog = datalog.clone();

    if skip_current {
        // Seek to end first before starting the thread
        reader.seek(std::io::SeekFrom::End(0)).unwrap();
    } else {
        // Queue an update command so that the reading thread immediately starts
        sender.send(LogReaderCommand::Update).unwrap();
    }

    std::thread::spawn(move || {
        let mut cache_string = String::new();
        let mut cache_events = Vec::new();
        loop {
            match rx.recv() {
                Ok(LogReaderCommand::Update) => {
                    while reader.read_line(&mut cache_string).is_ok_and(|size| size != 0) {
                        if check_exit(&rx) {
                            return
                        }
                        // TODO: probably use fetch-update
                        if status.load(Ordering::Relaxed) != LogReaderStatus::Initializing {
                            status.store(LogReaderStatus::Reading, Ordering::Relaxed);
                        }
                        if let Some(event) = parser.parse_line(cache_string.as_str()) {
                            cache_events.push(event);
                        }
                        cache_string.clear();
                    }

                    // Events collected, now acquire write lock to update the datalog
                    {
                        let mut datalog = datalog.write().unwrap();
                        for event in cache_events.drain(..) {
                            if check_exit(&rx) {
                                return;
                            }
                            datalog.handle_event(event);
                        }
                    }

                    status.store(LogReaderStatus::Idle, Ordering::Relaxed);
                },
                Ok(LogReaderCommand::Stop) => {
                    log::debug!("stop command received, closing logging thread");
                    return
                },
                Err(e) => {
                    log::error!("Error receiving from channel inside logging thread {e:?}");
                    return
                },
            }
        }
    })
}

impl LogReaderState {
    fn new(filepath: &str, poll_duration: Duration, skip_current: bool) -> Self {
        let (sender, rx) = std::sync::mpsc::channel();

        let datalog = Arc::new(RwLock::new(DataLog::new()));
        let status = Arc::new(AtomicLogReaderStatus::new(LogReaderStatus::Initializing));
        let datalog_thread = Some(init_datalog_thread(filepath, status.clone(), sender.clone(), rx, datalog.clone(), skip_current));
        let _watcher = start_watcher(sender.clone(), status.clone(), filepath, poll_duration);

        Self {
            sender,
            datalog,
            status,
            datalog_thread,
            _watcher,
        }
    }
}

impl LogReader {
    pub fn new(filepath: String, poll_duration: Duration, skip_current: bool) -> Self {
        let state = LogReaderState::new(&filepath, poll_duration, skip_current);

        Self {
            state,
            filepath,
            poll_duration,
        }
    }

    pub fn get_datalog(&self) -> Arc<RwLock<DataLog>> {
        self.state.datalog.clone()
    }

    /// Reset the logreader. Clear the datalog, restart the watcher and logging thread.
    pub fn reset(&mut self) {
        self.cleanup();

        let state = LogReaderState::new(&self.filepath, self.poll_duration, false);

        self.state = state;
    }

    pub fn get_status(&self) -> LogReaderStatus {
        self.state.status.load(Ordering::Relaxed)
    }

    fn cleanup(&mut self) {
        self.state.sender.send(LogReaderCommand::Stop).ok(); // Thread may already have exited, ignore send errors here
        if let Some(thread) = self.state.datalog_thread.take() {
            log::debug!("waiting on parser thread to close...");
            thread.join().unwrap();
            log::debug!("parser thread closed!")
        }
    }
}

impl Drop for LogReader {
    fn drop(&mut self) {
        self.cleanup();
    }
}
