//! Crate for easily creating system bars/panels/docks.
//!
//! The goal of this crate is to make it as simple as possible to create complex bars/panels/docks
//! for linux without having to worry about anything but rendering.
//!
//! To get started with the crate, a new bar needs to be created. This is done using the [`load`]
//! method in the [`Bar`]. Once this is acquired the [`recv`], [`try_recv`] and [`lock`] methods
//! should be all that is required to receive events and render the bar.
//!
//! [`Bar`]: struct.Bar.html
//! [`load`]: struct.Bar.html#method.load
//! [`recv`]: struct.Bar.html#method.recv
//! [`try_recv`]: struct.Bar.html#method.try_recv
//! [`lock`]: struct.Bar.html#method.lock
//!
//! # Examples
//!
//! ```no_run
//! use std::io::Cursor;
//!
//! use bar_config::Bar;
//!
//! fn main() {
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
//!     let mut bar = Bar::load(input).unwrap();
//!
//!     print_bar(&bar);
//!     loop {
//!         if let Ok(_) = bar.recv() {
//!             print_bar(&bar);
//!         }
//!     }
//! }
//!
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

pub use crate::bar::*;
