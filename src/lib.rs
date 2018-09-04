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
//! use bar_config::Bar;
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
//!     let config = bar.lock();
//!     for comp in config
//!         .left
//!         .iter()
//!         .chain(&config.center)
//!         .chain(&config.right)
//!     {
//!         if let Some(text) = comp.text() {
//!             print!("{}\t", text);
//!         }
//!     }
//!     println!("");
//! }
//! ```

#[macro_use]
extern crate serde_derive;

mod bar;
mod components;
pub mod config;
pub mod event;

pub use crate::bar::*;
