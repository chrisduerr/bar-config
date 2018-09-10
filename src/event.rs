//! Data models used for sending events to the config.
//!
//! This module provides everything necessary for sending events from the bar frontend to the
//! configuration. Implementing all of these events is necessary to make sure that all components
//! will function properly.
//!
//! # Examples
//!
//! Here's a minimal example for sending events to the config:
//! ```
//! use bar_config::event::{Event, Point};
//! use bar_config::bar::Bar;
//! use std::io::Cursor;
//!
//! let config_file = Cursor::new(String::from(
//!     "height: 30\n\
//!      monitors:\n\
//!       - { name: \"DVI-1\" }"
//! ));
//!
//! let mut bar = Bar::load(config_file).unwrap();
//! bar.notify(Event::MouseMotion(Point { x: 0, y: 0 }));
//! ```

use crate::components::ComponentID;

/// Event which can be transmitted to the components.
///
/// This event needs to be created from the frontend and can then be sent to the bar using
/// the [`Bar::notify`] method. Every component has the choice to use an event or ignore it.
///
/// [`Bar::notify`]: struct.Bar.html#method.notify
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub enum Event {
    /// Mouse button action anywhere on the screen.
    ///
    /// This event notifies all components that the user has clicked anywhere on the screen.
    /// It is required that the component knows about its position to act upon this event.
    /// To let a component know about its current position, the [`PositionChange`] event
    /// can be used.
    ///
    /// [`PositionChange`]: enum.Event.html#variant.PositionChange
    Click(MouseButton, MouseButtonState, Point),

    /// Update mouse position.
    ///
    /// This event notifies all components about the current position of the mouse on the screen.
    /// It is required that the component knows about its position to act upon this event.
    /// To let a component know about its current position, the [`PositionChange`] event
    /// can be used.
    ///
    /// [`PositionChange`]: enum.Event.html#variant.PositionChange
    MouseMotion(Point),

    /// Update the position of a component.
    ///
    /// This event is used to make a component aware of its position on the screen. This is
    /// required to react upon other events which are position dependent.
    PositionChange(ComponentPosition),
}

/// Button on the mouse.
///
/// This is required for the [`Click`] event.
///
/// [`Click`]: enum.Event.html#variant.Click
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub enum MouseButton {
    Left,
    Center,
    Right,
    WheelUp,
    WheelDown,
}

/// Mouse button states.
///
/// This is required for the [`Click`] event.
///
/// [`Click`]: enum.Event.html#variant.Click
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub enum MouseButtonState {
    Pressed,
    Released,
}

/// Exact position of a component on the screen.
///
/// This must be used in the [`PositionChange`] event which notifies components about their position
/// on the screen.
///
/// [`PositionChange`]: enum.Event.html#variant.PositionChange
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub struct ComponentPosition {
    pub comp_id: ComponentID,
    pub min_x: usize,
    pub max_x: usize,
    pub min_y: usize,
    pub max_y: usize,
}

/// Point on the screen.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}
