//! Bar configuration state.
//!
//! This module contains everything required to express the any state of the bar. The root
//! element ([`Config`]) can be accessed through the [`Bar`] using the [`load`] and [`lock`] methods.
//!
//! # Examples
//!
//! ```
//! use bar_config::Bar;
//! use std::io::Cursor;
//!
//! let config_file = Cursor::new(String::from(
//!     "height: 30\n\
//!      monitors:\n\
//!       - { name: \"DVI-1\" }"
//! ));
//!
//! let bar = Bar::load(config_file).unwrap();
//! let config = bar.lock();
//!
//! assert_eq!(config.height, 30);
//! assert_eq!(config.monitors.len(), 1);
//! assert_eq!(config.monitors[0].name, "DVI-1");
//! ```
//!
//! [`Config`]: struct.Config.html
//! [`Bar`]: ../struct.Bar.html
//! [`load`]: ../struct.Bar.html#method.load
//! [`lock`]: ../struct.Bar.html#method.lock

use serde::de::{Deserializer, Error};
use serde::Deserialize;

use std::path::{Path, PathBuf};

use crate::components::Component;

/// Root element of the bar configuration.
///
/// This element contains the complete state of the bar necessary to render it.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub height: u8,
    pub position: Option<Position>,
    pub background: Option<Background>,
    #[serde(
        deserialize_with = "deserialize_monitors",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub monitors: Vec<Monitor>,
    pub defaults: Option<ComponentSettings>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub left: Vec<Box<Component>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub center: Vec<Box<Component>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub right: Vec<Box<Component>>,
}

// Require at least one monitor
fn deserialize_monitors<'a, D>(deserializer: D) -> Result<Vec<Monitor>, D::Error>
where
    D: Deserializer<'a>,
{
    match Vec::<Monitor>::deserialize(deserializer) {
        Ok(monitors) => {
            if monitors.is_empty() {
                Err(D::Error::custom(String::from(
                    "at least one monitor is required",
                )))
            } else {
                Ok(monitors)
            }
        }
        err => err,
    }
}

/// Default options available for every component.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize)]
pub struct ComponentSettings {
    pub foreground: Option<Color>,
    pub background: Option<Background>,
    pub width: Option<u8>,
    pub padding: Option<u8>,
    pub offset_x: Option<i8>,
    pub offset_y: Option<i8>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fonts: Vec<Font>,
    pub border: Option<Border>,
}

/// Background of a component or the bar.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Background {
    Image(PathBuf),
    Color(Color),
}

impl<'de> Deserialize<'de> for Background {
    fn deserialize<D>(deserializer: D) -> Result<Background, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer) {
            Ok(text) => {
                if text.starts_with('#') {
                    Color::from_str(&text)
                        .map_err(D::Error::custom)
                        .map(Background::Color)
                } else {
                    Path::new(&text)
                        .canonicalize()
                        .map_err(D::Error::custom)
                        .map(Background::Image)
                }
            }
            Err(err) => Err(err),
        }
    }
}

/// Distinct identification for a font.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize)]
pub struct Font {
    pub description: String,
    pub size: u8,
}

/// Distinct identification for a monitor.
///
/// The [`fallback_names`] can be used to specify alternative screens which should be used when the
/// primary monitor is not available.
///
/// [`fallback_names`]: #structfield.fallback_names
#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize)]
pub struct Monitor {
    pub name: String,
    #[serde(default)]
    pub fallback_names: Vec<String>,
}

/// Border separating the bar from the rest of the WM.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Deserialize)]
pub struct Border {
    pub height: u8,
    pub color: Color,
}

/// Available positions for the bar.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Deserialize)]
pub enum Position {
    Top,
    Bottom,
}

/// RGBA color specified as four values from 0 to 255.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    // Deserialize the `#ff00ff` and `#ff00ff00` color formats
    fn from_str(string: &str) -> Result<Self, String> {
        if !string.starts_with('#') || (string.len() != 7 && string.len() != 9) {
            return Err(String::from(
                "colors need to follow the format `#RRGGBB` or `#RRGGBBAA`",
            ));
        }

        let radix_error = |_| String::from("hexadecimal color digits need to be within the range 0..=F");
        let r = u8::from_str_radix(&string[1..3], 16).map_err(radix_error)?;
        let g = u8::from_str_radix(&string[3..5], 16).map_err(radix_error)?;
        let b = u8::from_str_radix(&string[5..7], 16).map_err(radix_error)?;
        let a = if string.len() == 9 {
            u8::from_str_radix(&string[7..9], 16).map_err(radix_error)?
        } else {
            255
        };

        Ok(Color::new(r, g, b, a))
    }
}

// Format the color in the format `#RRGGBBAA`
impl ToString for Color {
    fn to_string(&self) -> String {
        format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer) {
            Ok(color_string) => Color::from_str(&color_string).map_err(D::Error::custom),
            Err(err) => Err(err),
        }
    }
}
