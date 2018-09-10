//! Components inside the bar.
//!
//! This module contains everything necessary for representing the state of a component in the bar.
//!
//! The main way to retrieve information about a component is by using the methods provided by the
//! [`Component`] trait.
//!
//! [`Component`]: trait.Component.html

mod clock;
mod undynamic;

use tokio::prelude::stream::{self, Stream};

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::components::clock::Clock;
use crate::components::undynamic::Undynamic;
use crate::config::Component as ConfigComponent;
use crate::event::Event;

pub use crate::config::{ComponentSettings, Font};

static COMPONENT_INDEX: AtomicUsize = AtomicUsize::new(0);

pub(crate) type ComponentStream = Box<Stream<Item = ComponentID, Error = ()> + Send>;

/// Unique component identifier.
///
/// This component identifier is automatically generated for each instance of a component at
/// startup. Two components of the same type will always have a different ID, however a component
/// will never change its ID. This allows identifying every component at any time solely through
/// its `ComponentID`.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub struct ComponentID(usize);

impl Default for ComponentID {
    fn default() -> Self {
        ComponentID(COMPONENT_INDEX.fetch_add(1, Ordering::Relaxed))
    }
}

trait ComponentTrait: Send {
    fn id(&self) -> ComponentID;

    fn text(&self) -> String;

    fn settings(&self) -> &ComponentSettings;

    #[doc(hidden)]
    fn stream(&self) -> ComponentStream {
        Box::new(stream::empty())
    }

    #[doc(hidden)]
    fn update(&mut self) -> bool {
        false
    }

    fn notify(&mut self, _event: Event) -> bool {
        false
    }
}

/// A single component inside the bar.
///
/// This struct represents a component/block/item/element inside the bar. All data required for
/// drawing this component can be acquired with the [`text`] and [`settings`] methods.
///
/// For components to act appropriately based on user interactions with the frontend, it is
/// required that the [`notify`] method will be supplied with all available events.
///
/// [`text`]: #method.text
/// [`settings`]: #method.settings
/// [`notify`]: #method.notify
pub struct Component(Box<ComponentTrait>);

impl Component {
    /// Return the unique identifier of this component.
    ///
    /// Since all updates received by the [`recv`] and [`try_recv`] methods return component IDs,
    /// this can be used to check which component needs to be updated, if a complete redraw is not
    /// desired.
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
    ///
    /// [`recv`]: ../bar/struct.Bar.html#method.recv
    /// [`try_recv`]: ../bar/struct.Bar.html#method.try_recv
    pub fn id(&self) -> ComponentID {
        self.0.id()
    }

    /// Get component text.
    ///
    /// This will query a component for the text that should be displayed on the component at this
    /// time. If there is no text that should be rendered, an empty String will be returned.
    ///
    /// To get information about the font and color of the text, the [`settings`] query can be
    /// used.
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
    ///       - { text: \"hello\" }"
    /// ));
    ///
    /// let mut bar = Bar::load(config_file).unwrap();
    /// let text = bar.components()[0].text();
    ///
    /// assert_eq!(text, String::from("hello"));
    /// ```
    ///
    /// [`settings`]: #method.settings
    pub fn text(&self) -> String {
        self.0.text()
    }

    /// Get component settings.
    ///
    /// This will query the component for the settings which should be used to render it. The full
    /// list of data returned by this query, can be found in the [`ComponentSettings`]
    /// documentation.
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
    ///       - { text: \"hello\", width: 99 }"
    /// ));
    ///
    /// let mut bar = Bar::load(config_file).unwrap();
    /// let settings = bar.components()[0].settings();
    ///
    /// assert_eq!(settings.width, Some(99));
    /// ```
    ///
    /// [`ComponentSettings`]: struct.ComponentSettings.html
    pub fn settings(&self) -> &ComponentSettings {
        self.0.settings()
    }

    /// Notify all components about a frontend event.
    ///
    /// Since this crate does not provide any functionality to deal with the rendering of a bar, it
    /// is required to pass events to the components to make sure they can react upon them.
    ///
    /// All available events can be found in the documentation of the [`Event`] enum.
    ///
    /// To ensure that all components work properly, it is required that all events available in
    /// the [`Event`] enum are propagated properly.
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
    /// [`Event`]: ../event/enum.Event.html
    pub fn notify(&mut self, event: Event) -> bool {
        self.0.notify(event)
    }

    pub(crate) fn stream(&self) -> ComponentStream {
        self.0.stream()
    }

    pub(crate) fn update(&mut self) -> bool {
        self.0.update()
    }
}

impl From<ConfigComponent> for Component {
    fn from(comp: ConfigComponent) -> Self {
        match comp.name.as_str() {
            "clock" => Clock::create(comp.settings, comp.extra),
            _ => Undynamic::create(comp.settings, comp.extra),
        }
    }
}
