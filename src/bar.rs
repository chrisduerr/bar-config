//! Module for the bar state.
//!
//! This module contains the main components necessary to create and update a bar.

#[cfg(all(feature = "json-fmt", not(feature = "toml-fmt")))]
use serde_json as serde_fmt;
#[cfg(not(any(feature = "toml-fmt", feature = "json-fmt")))]
use serde_yaml as serde_fmt;
#[cfg(all(feature = "toml-fmt", not(feature = "json-fmt")))]
use toml as serde_fmt;

use tokio::prelude::stream::{self, Stream};

use std::io::{Error as IOError, ErrorKind, Read};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;

use crate::components::{Component, ComponentID, ComponentStream};
use crate::config::{Background, Component as ConfigComponent, Config};
use crate::event::Event;

pub use crate::config::{Border, Monitor, Position};

/// Data model for the bar state.
///
/// The `Bar` is the main data model used to represent the state of the bar at any point. A new
/// `Bar` can be created by loading it from a configuration file using the [`load`] method.
///
/// Using the `Bar` struct, it is possible to query for updates using the [`recv`] and [`try_recv`]
/// methods. These will return the ID of the component which has been updated.
///
/// To access any component, the [`left`], [`center`], [`right`], and [`components`] methods can be
/// used.
///
/// It is required to make use of the [`notify`] method to let components know about updates to the
/// frontend of the bar.
///
/// [`load`]: #method.load
/// [`left`]: #method.left
/// [`center`]: #method.center
/// [`right`]: #method.right
/// [`notify`]: #method.notify
/// [`recv`]: #method.recv
/// [`try_recv`]: #method.try_recv
/// [`components`]: #method.components
pub struct Bar {
    general: General,
    left: Vec<Component>,
    center: Vec<Component>,
    right: Vec<Component>,
    events: Option<(Sender<ComponentID>, Receiver<ComponentID>)>,
}

/// General bar settings.
///
/// The general settings are used to setup the bar. These will never change during the runtime of
/// the bar and are only required for the initial setup.
pub struct General {
    pub height: u8,
    pub position: Position,
    pub background: Background,
    pub border: Option<Border>,
    pub monitors: Vec<Monitor>,
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
    /// use bar_config::bar::Bar;
    /// use std::io::Cursor;
    ///
    /// let config_file = Cursor::new(String::from(
    ///     "height: 30\n\
    ///      monitors:\n\
    ///       - { name: \"DVI-1\" }"
    /// ));
    ///
    /// let bar = Bar::load(config_file).unwrap();
    ///
    /// assert_eq!(bar.general().height, 30);
    /// assert_eq!(bar.general().monitors.len(), 1);
    /// assert_eq!(bar.general().monitors[0].name, "DVI-1");
    /// ```
    ///
    /// [`io::ErrorKind::InvalidData`]:
    /// https://doc.rust-lang.org/std/io/enum.ErrorKind.html#variant.InvalidData
    /// [`recv`]: #method.recv
    /// [`try_recv`]: #method.try_recv
    pub fn load<T: Read>(mut config_file: T) -> Result<Self, IOError> {
        let mut content = String::new();
        config_file.read_to_string(&mut content)?;

        let config: Config =
            serde_fmt::from_str(&content).map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;

        let general = General {
            height: config.height,
            position: config.position,
            background: config.background,
            border: config.border,
            monitors: config.monitors,
        };

        // Convert component struct to trait and set general fallbacks
        let defaults = config.defaults.clone();
        let convert = |mut comps: Vec<ConfigComponent>| {
            comps
                .drain(..)
                .map(|mut c| {
                    c.settings.fallback(&defaults);
                    c.into()
                }).collect()
        };
        let left = convert(config.left);
        let center = convert(config.center);
        let right = convert(config.right);

        Ok(Self {
            general,
            left,
            center,
            right,
            events: None,
        })
    }

    /// Blocking poll for updates.
    ///
    /// Polls the event buffer for the next event. If no event is currently queued, this will block
    /// until the next event is received.
    ///
    /// # Examples
    ///
    /// ```
    /// use bar_config::bar::Bar;
    /// use std::io::Cursor;
    ///
    /// let config_file = Cursor::new(String::from(
    ///     "height: 30\n\
    ///      monitors:\n\
    ///       - { name: \"DVI-1\" }\n\
    ///      left:\n\
    ///       - { name: \"clock\", interval: 1 }"
    /// ));
    ///
    /// let mut bar = Bar::load(config_file).unwrap();
    /// let component_id = bar.recv();
    ///
    /// println!("Component {:?} was updated!", component_id);
    /// ```
    pub fn recv(&mut self) -> ComponentID {
        if self.events.is_none() {
            self.events = Some(self.start_loop());
        }

        // Process updates until the first dirty component is found
        loop {
            let comp_id = self.events.as_ref().unwrap().1.recv().unwrap();
            if self.update_component(comp_id) {
                return comp_id;
            }
        }
    }

    /// Non-Blocking poll for updates.
    ///
    /// Polls the event buffer for the next event. If no event is currently queued, this will
    /// return `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bar_config::bar::Bar;
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
    /// let update = bar.try_recv();
    ///
    /// if let Some(component_id) = update {
    ///     println!("Component {:?} was updated!", component_id);
    /// } else {
    ///     println!("No new event!");
    /// }
    /// ```
    pub fn try_recv(&mut self) -> Option<ComponentID> {
        if self.events.is_none() {
            self.events = Some(self.start_loop());
        }

        // Process updates until the first dirty component is found
        loop {
            match self.events.as_ref().unwrap().1.try_recv() {
                Ok(comp_id) => {
                    if self.update_component(comp_id) {
                        return Some(comp_id);
                    }
                }
                Err(TryRecvError::Empty) => return None,
                Err(e) => return Err(e).unwrap(),
            }
        }
    }

    // Update the component with the matching ID
    fn update_component(&mut self, comp_id: ComponentID) -> bool {
        for comp in self.components_mut() {
            if comp.id() == comp_id {
                return comp.update();
            }
        }
        false
    }

    /// General bar settings.
    ///
    /// These settings store all settings that are not directly associated to any component. This
    /// should only be required during startup, since it is never modified at runtime.
    ///
    /// # Examples
    ///
    /// ```
    /// use bar_config::bar::Bar;
    /// use std::io::Cursor;
    ///
    /// let config_file = Cursor::new(String::from(
    ///     "height: 30\n\
    ///      monitors:\n\
    ///       - { name: \"DVI-1\" }"
    /// ));
    ///
    /// let mut bar = Bar::load(config_file).unwrap();
    /// let general = bar.general();
    ///
    /// assert_eq!(general.height, 30);
    /// ```
    pub fn general(&self) -> &General {
        &self.general
    }

    /// Left bar components.
    ///
    /// Vector with all components which should be rendered at the left side of the bar. These
    /// could change at any time, so this can be used after receiving an update to redraw all
    /// components with the same alignment.
    ///
    /// # Examples
    ///
    /// ```
    /// use bar_config::bar::Bar;
    /// use std::io::Cursor;
    ///
    /// let config_file = Cursor::new(String::from(
    ///     "height: 30\n\
    ///      monitors:\n\
    ///       - { name: \"DVI-1\" }\n\
    ///      left:\n\
    ///       - { text: \"test\" }"
    /// ));
    ///
    /// let mut bar = Bar::load(config_file).unwrap();
    /// let left = bar.left();
    ///
    /// assert_eq!(left[0].text(), String::from("test"));
    /// ```
    pub fn left(&self) -> &Vec<Component> {
        &self.left
    }

    /// Center bar components.
    ///
    /// Vector with all components which should be rendered at the center of the bar. These
    /// could change at any time, so this can be used after receiving an update to redraw all
    /// components with the same alignment.
    ///
    /// # Examples
    ///
    /// ```
    /// use bar_config::bar::Bar;
    /// use std::io::Cursor;
    ///
    /// let config_file = Cursor::new(String::from(
    ///     "height: 30\n\
    ///      monitors:\n\
    ///       - { name: \"DVI-1\" }\n\
    ///      center:\n\
    ///       - { text: \"test\" }"
    /// ));
    ///
    /// let mut bar = Bar::load(config_file).unwrap();
    /// let center = bar.center();
    ///
    /// assert_eq!(center[0].text(), String::from("test"));
    /// ```
    pub fn center(&self) -> &Vec<Component> {
        &self.center
    }

    /// Right bar components.
    ///
    /// Vector with all components which should be rendered at the right side of the bar. These
    /// could change at any time, so this can be used after receiving an update to redraw all
    /// components with the same alignment.
    ///
    /// # Examples
    ///
    /// ```
    /// use bar_config::bar::Bar;
    /// use std::io::Cursor;
    ///
    /// let config_file = Cursor::new(String::from(
    ///     "height: 30\n\
    ///      monitors:\n\
    ///       - { name: \"DVI-1\" }\n\
    ///      right:\n\
    ///       - { text: \"test\" }"
    /// ));
    ///
    /// let mut bar = Bar::load(config_file).unwrap();
    /// let right = bar.right();
    ///
    /// assert_eq!(right[0].text(), String::from("test"));
    /// ```
    pub fn right(&self) -> &Vec<Component> {
        &self.right
    }

    /// All bar components.
    ///
    /// Vector with all components of the bar. This can be used for performing actions on all
    /// components independent of component alignment.
    ///
    /// # Examples
    ///
    /// ```
    /// use bar_config::bar::Bar;
    /// use std::io::Cursor;
    ///
    /// let config_file = Cursor::new(String::from(
    ///     "height: 30\n\
    ///      monitors:\n\
    ///       - { name: \"DVI-1\" }\n\
    ///      right:\n\
    ///       - { text: \"right\" }\n\
    ///      center:\n\
    ///       - { text: \"center\" }"
    /// ));
    ///
    /// let mut bar = Bar::load(config_file).unwrap();
    /// let components = bar.components();
    ///
    /// assert_eq!(components.len(), 2);
    /// ```
    pub fn components(&self) -> Vec<&Component> {
        self.left
            .iter()
            .chain(&self.center)
            .chain(&self.right)
            .collect()
    }

    fn components_mut(&mut self) -> Vec<&mut Component> {
        self.left
            .iter_mut()
            .chain(&mut self.center)
            .chain(&mut self.right)
            .collect()
    }

    /// Send an event to all components.
    ///
    /// Notifies all components that a new event is available. The components then have the choice
    /// to react upon the event or ignore it completely.
    ///
    /// If a component handles the event and marks itself as `dirty` as a result of the event, a
    /// new redraw request will be queued for the [`recv`] and [`try_recv`] methods.
    ///
    /// # Examples
    ///
    /// ```
    /// use bar_config::event::{Event, Point};
    /// use bar_config::bar::Bar;
    /// use std::io::Cursor;
    ///
    /// let config_file = Cursor::new(String::from(
    ///     "height: 30\n\
    ///      monitors:\n\
    ///       - { name: \"DVI-1\" }"
    /// ));
    ///
    /// let mut bar = Bar::load(config_file).unwrap();
    /// bar.notify(Event::MouseMotion(Point { x: 0, y: 0 }));
    /// ```
    ///
    /// [`recv`]: #method.recv
    /// [`try_recv`]: #method.try_recv
    pub fn notify(&mut self, event: Event) {
        // Find all dirty components
        let mut dirty_comps = Vec::new();
        for comp in self.components_mut() {
            if comp.notify(event) {
                dirty_comps.push(comp.id());
            }
        }

        if let Some((ref events_tx, _)) = self.events {
            for comp_id in dirty_comps {
                events_tx.send(comp_id).unwrap();
            }
        }
    }

    // Starts the event loop in a new thread
    fn start_loop(&self) -> (Sender<ComponentID>, Receiver<ComponentID>) {
        let (events_tx, events_rx) = mpsc::channel();
        let bar_events_tx = events_tx.clone();

        // Combine all component events into one blocking event stream
        let mut combined: ComponentStream = Box::new(stream::empty());
        for comp in self.components() {
            combined = Box::new(combined.select(comp.stream()));
        }

        thread::spawn(move || {
            // Propagate events to main thread
            let combined = combined.for_each(move |comp_id| {
                events_tx.send(comp_id).unwrap();
                Ok(())
            });

            // Iterate over all component events forever
            tokio::run(combined);
        });

        (bar_events_tx, events_rx)
    }
}
