#[cfg(all(feature = "json-fmt", not(feature = "toml-fmt")))]
use serde_json as serde_fmt;
#[cfg(not(any(feature = "toml-fmt", feature = "json-fmt")))]
use serde_yaml as serde_fmt;
#[cfg(all(feature = "toml-fmt", not(feature = "json-fmt")))]
use toml as serde_fmt;

use tokio::prelude::stream::{self, Stream};

use dirs;
use std::fs::File;
use std::io::{Error as IOError, ErrorKind, Read};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, RecvError, TryRecvError};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

use crate::components::{Component, ComponentID, ComponentStream};
use crate::config::Config;

const PATH_LOAD_ORDER: [&str; 3] = [
    "{config}/{name}.{ext}",
    "{home}/.{name}.{ext}",
    "/etc/{name}/{name}.{ext}",
];

/// Wrapper around the bar configuration.
///
/// This is a safe wrapper around the bar configuration. It can notify consumers about any updates
/// to the state of the configuration file.
///
/// The `Bar` is the central point of interaction for any consumer. The [`Config`] can be  accessed
/// through an instance of `Bar` using the [`load`] method. The [`recv`] and [`try_recv`] methods
/// should be used to check for updates of any component of the configuration file.
///
/// [`Config`]: config/struct.Config.html
/// [`load`]: #method.load
/// [`recv`]: #method.recv
/// [`try_recv`]: #method.try_recv
#[derive(Debug)]
pub struct Bar {
    config: Arc<Mutex<Config>>,
    events: Option<Receiver<ComponentID>>,
}

impl Bar {
    /// Load the initial bar configuration.
    ///
    /// Loads the initial state of the bar configuration from the specified source.
    ///
    /// The method will not launch any of the components that are specified in the configuration
    /// file, this is done with the [`recv`] and [`try_recv`] methods.
    ///
    /// # Errors
    ///
    /// If the `config_file` cannot be read or its content is not valid. If the configuration is
    /// invalid, the [`io::ErrorKind::InvalidData`] value is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use bar_config::Bar;
    /// use std::io::Cursor;
    ///
    /// let config_file = Cursor::new(String::from(
    ///     "height: 30\n\
    ///      monitors:\n\
    ///       - { name: \"DVI-1\" }"
    /// ));
    ///
    /// let bar = Bar::load(config_file).unwrap();
    /// let config = bar.lock();
    ///
    /// assert_eq!(config.height, 30);
    /// assert_eq!(config.monitors.len(), 1);
    /// assert_eq!(config.monitors[0].name, "DVI-1");
    /// ```
    ///
    /// [`io::ErrorKind::InvalidData`]:
    /// https://doc.rust-lang.org/std/io/enum.ErrorKind.html#variant.InvalidData
    /// [`recv`]: #method.recv
    /// [`try_recv`]: #method.try_recv
    pub fn load<T: Read>(mut config_file: T) -> Result<Self, IOError> {
        let mut content = String::new();
        config_file.read_to_string(&mut content)?;

        let config =
            serde_fmt::from_str(&content).map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;

        Ok(Bar {
            config: Arc::new(Mutex::new(config)),
            events: None,
        })
    }

    /// Blocking poll for updates.
    ///
    /// Polls the event buffer for the next event. If no event is currently queued, this will block
    /// until the next event is received.
    ///
    /// # Errors
    ///
    /// Returns an error if the event loop unexpectedly shut down. No further events will be
    /// received once this method has failed.
    ///
    /// # Examples
    ///
    /// ```
    /// use bar_config::Bar;
    /// use std::io::Cursor;
    ///
    /// let config_file = Cursor::new(String::from(
    ///     "height: 30\n\
    ///      monitors:\n\
    ///       - { name: \"DVI-1\" }\n\
    ///      left:\n\
    ///       - { name: \"clock\" }"
    /// ));
    ///
    /// let mut bar = Bar::load(config_file).unwrap();
    /// if let Ok(component_id) = bar.recv() {
    ///     println!("Component {:?} was updated!", component_id);
    /// }
    /// ```
    pub fn recv(&mut self) -> Result<ComponentID, RecvError> {
        if self.events.is_none() {
            self.events = Some(self.start_loop());
        }

        self.events.as_ref().unwrap().recv()
    }

    /// Non-Blocking poll for updates.
    ///
    /// Polls the event buffer for the next event. If no event is currently queued, this will
    /// return an error using the [`TryRecvError::Empty`] variant.
    ///
    /// # Errors
    ///
    /// Returns an error if the event loop unexpectedly shut down. No further events will be
    /// received once this method has failed.
    ///
    /// # Examples
    ///
    /// ```
    /// use bar_config::Bar;
    /// use std::io::Cursor;
    ///
    /// let config_file = Cursor::new(String::from(
    ///     "height: 30\n\
    ///      monitors:\n\
    ///       - { name: \"DVI-1\" }\n\
    ///      left:\n\
    ///       - { name: \"clock\" }"
    /// ));
    ///
    /// let mut bar = Bar::load(config_file).unwrap();
    /// if let Ok(component_id) = bar.try_recv() {
    ///     println!("Component {:?} was updated!", component_id);
    /// }
    /// ```
    ///
    /// [`TryRecvError::Empty`]:
    /// https://doc.rust-lang.org/nightly/std/sync/mpsc/enum.TryRecvError.html#variant.Empty
    pub fn try_recv(&mut self) -> Result<ComponentID, TryRecvError> {
        if self.events.is_none() {
            self.events = Some(self.start_loop());
        }

        self.events.as_ref().unwrap().try_recv()
    }

    /// Lock the configuration file.
    ///
    /// Locks the configuration file so its state can be used to render the bar. Since this creates
    /// a `MutexGuard`, no events will be received while the lock is held.
    ///
    /// # Examples
    /// ```
    /// use bar_config::Bar;
    /// use std::io::Cursor;
    ///
    /// let config_file = Cursor::new(String::from(
    ///     "height: 30\n\
    ///      monitors:\n\
    ///       - { name: \"DVI-1\" }"
    /// ));
    ///
    /// let mut bar = Bar::load(config_file).unwrap();
    /// let config = bar.lock();
    ///
    /// assert_eq!(config.height, 30);
    /// assert_eq!(config.monitors.len(), 1);
    /// assert_eq!(config.monitors[0].name, "DVI-1");
    /// ```
    pub fn lock(&self) -> MutexGuard<Config> {
        self.config.lock().unwrap()
    }

    // Starts the event loop in a new thread
    fn start_loop(&self) -> Receiver<ComponentID> {
        let (events_tx, events_rx) = mpsc::channel();

        let config = self.config.clone();
        thread::spawn(move || {
            // Combine all component events into one blocking iterator
            let combined = {
                let config = config.lock().unwrap();
                let mut combined: ComponentStream = Box::new(stream::empty());
                for comp in config
                    .left
                    .iter()
                    .chain(&config.center)
                    .chain(&config.right)
                {
                    combined = Box::new(combined.select(comp.stream()));
                }
                combined
            };

            // Propagate events
            let combined = combined.for_each(move |comp_id| {
                let mut config = config.lock().unwrap();

                // Try to find the component with a matching ID and update it
                let update_comps = |comps: &mut Vec<Box<Component>>| {
                    if let Some(true) = comps
                        .iter_mut()
                        .find(|comp| comp_id == comp.id())
                        .map(|comp| comp.update())
                    {
                        events_tx.send(comp_id).unwrap();
                        true
                    } else {
                        false
                    }
                };

                // Short-circuit update the component with a matching ID
                let _ = update_comps(&mut config.left)
                    || update_comps(&mut config.center)
                    || update_comps(&mut config.right);

                Ok(())
            });

            // Iterate over all component events forever
            tokio::run(combined);
        });

        events_rx
    }
}

/// Find the configuration file.
///
/// This looks for the configuration file of the bar in a predefined list of directories.
/// The `name` parameter is used for the configuration file name and the extension is based
/// on the enabled features.
///
/// The directories are used in the following order:
/// ```text
/// ~/.config/name.ext
/// ~/.name.ext
/// /etc/name/name.ext
/// ```
///
/// The file endings map to the specified library features:
///
/// Feature  | Extension
/// ---------|----------
/// default  | yml
/// toml-fmt | toml
/// json-fmt | json
///
/// # Errors
///
/// This method will fail if the configuration file cannot be opened. If there was no file present
/// in any of the directories, the [`io::ErrorKind::NotFound`] error will be returned.
///
/// # Examples
///
/// ```
/// use bar_config::config_file;
/// use std::io::ErrorKind;
///
/// let file_result = config_file("mybar");
/// assert_eq!(file_result.err().unwrap().kind(), ErrorKind::NotFound);
/// ```
///
/// [`io::ErrorKind::NotFound`]: https://doc.rust-lang.org/std/io/enum.ErrorKind.html#variant.NotFound
pub fn config_file(name: &str) -> Result<File, IOError> {
    for path in &PATH_LOAD_ORDER[..] {
        let mut path = path.to_string();
        #[allow(ifs_same_cond)]
        let extension = if cfg!(feature = "toml-fmt") && !cfg!(feature = "json-fmt") {
            "toml"
        } else if cfg!(feature = "json-fmt") && !cfg!(feature = "toml-fmt") {
            "json"
        } else {
            "yml"
        };
        path = path.replace("{ext}", extension);
        path = path.replace(
            "{home}",
            &dirs::home_dir()
                .and_then(|p| Some(p.to_string_lossy().to_string()))
                .unwrap_or_else(String::new),
        );
        path = path.replace(
            "{config}",
            &dirs::config_dir()
                .and_then(|p| Some(p.to_string_lossy().to_string()))
                .unwrap_or_else(String::new),
        );
        path = path.replace("{name}", name);

        let metadata = Path::new(&path).metadata();
        if let Ok(metadata) = metadata {
            if metadata.is_file() {
                return Ok(File::open(path)?);
            }
        }
    }
    Err(IOError::new(ErrorKind::NotFound, "no config file present"))
}
