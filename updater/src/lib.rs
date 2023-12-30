use std::sync::{Arc, Mutex};

use self_update::update::ReleaseUpdate;

#[derive(Default, Debug, Clone)]
pub enum UpdateStatus {
    #[default]
    Idle,
    Fetching,
    Fetched(VersionStatus),
    Updating,
    Updated,
    Error(String),
}

#[derive(Debug, Clone)]
pub enum VersionStatus {
    UpToDate,
    Update(String, Option<String>),
}

#[derive(Default)]
pub struct Updater {
    pub current_version: String,
    pub status: Arc<Mutex<UpdateStatus>>,
    options: UpdaterOptions
}

#[derive(Default, Clone)]
pub struct UpdaterOptions {
    no_confirm: bool,
    show_download_progress: bool,
}

impl UpdaterOptions {
    pub fn no_confirm(self, no_confirm: bool) -> Self {
        Self {
            no_confirm,
            ..self
        }
    }

    pub fn show_download_progress(self, show_download_progress: bool) -> Self {
        Self {
            show_download_progress,
            ..self
        }
    }
}

impl Updater {
    pub fn init(&mut self, version: String) {
        self.current_version = version;
    }

    pub fn set_options(&mut self, options: UpdaterOptions) {
        self.options = options;
    }

    fn build_update(current_version: &str, options: UpdaterOptions) -> Box<dyn ReleaseUpdate> {
        self_update::backends::github::Update::configure()
            // TODO: Consider metadata-ing these
            .repo_owner("richteer")
            .repo_name("inkbound-dps")
            .bin_name("inkbound-dps")
            .show_download_progress(options.show_download_progress)
            .no_confirm(options.no_confirm)
            .current_version(current_version)
            .build().unwrap()
    }

    pub fn fetch_update(&mut self, sync: bool) {
        assert!(!self.current_version.is_empty());

        let status = self.status.clone();
        {
            let mut status = status.lock().unwrap();
            match *status {
                UpdateStatus::Fetching | UpdateStatus::Updating => {
                    log::debug!("Updater busy, ignoring fetch request");
                    return;
                }
                _ => {
                    *status = UpdateStatus::Fetching;
                },
            };
        }

        let current_version = self.current_version.clone();
        let options = self.options.clone();

        let handle = std::thread::spawn(move || {
            let update = Self::build_update(&current_version, options);

            let new_status = match update.get_latest_release() {
                Ok(r) =>
                    match self_update::version::bump_is_greater(&current_version, &r.version).unwrap_or(false) {
                        true => UpdateStatus::Fetched(VersionStatus::Update(r.version, r.body)),
                        false => UpdateStatus::Fetched(VersionStatus::UpToDate),
                    },
                Err(e) => UpdateStatus::Error(e.to_string()),
            };

            {
                let mut status = status.lock().unwrap();
                *status = new_status;
            }
        });

        if sync {
            handle.join().unwrap();
        }
    }

    pub fn do_update(&mut self, sync: bool) {
        assert!(!self.current_version.is_empty());

        let status = self.status.clone();
        {
            let mut status = status.lock().unwrap();
            match *status {
                UpdateStatus::Fetching | UpdateStatus::Updating => {
                    log::debug!("Updater busy, ignoring update request");
                    return;
                }
                _ => {
                    *status = UpdateStatus::Updating;
                },
            };
        }

        let current_version = self.current_version.clone();
        let options = self.options.clone();

        let handle = std::thread::spawn(move || {
            {
                let mut status = status.lock().unwrap();
                *status = UpdateStatus::Updating;
            }
            
            let update = Self::build_update(&current_version, options);

            let new_status = match update.update() {
                Ok(_) => UpdateStatus::Updated,
                Err(e) => UpdateStatus::Error(e.to_string()),
            };

            {
                let mut status = status.lock().unwrap();
                *status = new_status;
            }
        });

        if sync {
            handle.join().unwrap();
        }
    }
}

lazy_static::lazy_static! {
    pub static ref UPDATER: Arc<Mutex<Updater>> = Arc::new(Mutex::new(Updater::default()));
}
