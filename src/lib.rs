//! Crate for easily creating system bars/panels/docks.
//!
//! The goal of this crate is to make it as simple as possible to create complex bars/panels/docks
//! for linux without having to worry about anything but rendering.
//!
//! To get started with the crate, a new bar needs to be created. This is done using the [`load`]
//! method in the [`Bar`].
//!
//! [`Bar`]: struct.Bar.html
//! [`load`]: struct.Bar.html#method.load
//!
//! # Examples
//!
//! This example shows you how to create a simple bar with three components which prints the state
//! of every component in the console every time it's updated.
//!
//! ```no_run
//! use std::io::Cursor;
//!
//! use bar_config::bar::Bar;
//!
//! fn main() {
//!     // Create input configuration input with three components
//!     let input = Cursor::new(String::from(
//!         "\
//!          height: 30\n\
//!          monitors:\n\
//!          - { name: \"DVI-1\" }\n\
//!          left:\n\
//!          - { text: \"Hello, World!\" }\n\
//!          center:\n\
//!          - { name: \"clock\" }\n\
//!          right:\n\
//!          - { text: \"VOLUME\" }",
//!     ));
//!
//!     // Load the bar configuration from the input
//!     let mut bar = Bar::load(input).unwrap();
//!
//!     // Render the state of the bar at startup
//!     print_bar(&bar);
//!
//!     loop {
//!         // Wait for any update to the bar
//!         let _ = bar.recv();
//!
//!         // Print all components every time one changes
//!         print_bar(&bar);
//!     }
//! }
//!
//! // Prints the text of every component in the configuration
//! fn print_bar(bar: &Bar) {
//!     for comp in bar.components()
//!     {
//!         print!("{}\t", comp.text());
//!     }
//!     println!("");
//! }
//! ```

#![feature(tool_lints)]
#![deny(clippy::all)]

#[macro_use]
extern crate serde_derive;

mod config;

pub mod bar;
pub mod components;
pub mod event;

pub use crate::config::{Background, Color};
pub use image;

use dirs;

use std::fs::File;
use std::io::{Error as IOError, ErrorKind};
use std::path::Path;

const PATH_LOAD_ORDER: [&str; 3] = [
    "{config}/{name}.{ext}",
    "{home}/.{name}.{ext}",
    "/etc/{name}/{name}.{ext}",
];

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
        #[allow(clippy::ifs_same_cond)]
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
