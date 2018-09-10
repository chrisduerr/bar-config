#[cfg(all(feature = "json-fmt", not(feature = "toml-fmt")))]
use serde_json as serde_fmt;
#[cfg(not(any(feature = "toml-fmt", feature = "json-fmt")))]
use serde_yaml as serde_fmt;
#[cfg(all(feature = "toml-fmt", not(feature = "json-fmt")))]
use toml as serde_fmt;

use image::{self, DynamicImage};
use serde::de::{Deserializer, Error};
use serde::Deserialize;

use std::path::Path;

/// Root element of the bar configuration file.
#[derive(Deserialize)]
pub(crate) struct Config {
    pub height: u8,
    #[serde(default)]
    pub position: Position,
    #[serde(default)]
    pub background: Background,
    pub border: Option<Border>,
    #[serde(
        deserialize_with = "deserialize_monitors",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub monitors: Vec<Monitor>,
    #[serde(default)]
    pub defaults: ComponentSettings,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub left: Vec<Component>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub center: Vec<Component>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub right: Vec<Component>,
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

#[derive(Clone, Deserialize)]
pub(crate) struct Component {
    #[serde(default)]
    pub name: String,
    #[serde(flatten)]
    pub settings: ComponentSettings,
    #[serde(flatten)]
    pub extra: serde_fmt::Value,
}

/// Settings of a component.
///
/// These component settings represent most of the component's state required to draw it. All
/// components automatically inherit the default configuration options from the bar as fallbacks,
/// however all fields are still optional.
#[derive(Clone, Deserialize, Default)]
pub struct ComponentSettings {
    pub foreground: Option<Color>,
    pub background: Option<Background>,
    pub width: Option<u8>,
    pub padding: Option<u8>,
    pub offset_x: Option<i8>,
    pub offset_y: Option<i8>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fonts: Vec<Font>,
}

impl ComponentSettings {
    pub(crate) fn fallback(&mut self, fallback: &ComponentSettings) {
        fn select<T: Clone>(main: &mut Option<T>, fallback: &Option<T>) {
            if main.is_none() {
                *main = fallback.clone();
            }
        }

        select(&mut self.foreground, &fallback.foreground);
        select(&mut self.background, &fallback.background);
        select(&mut self.width, &fallback.width);
        select(&mut self.padding, &fallback.padding);
        select(&mut self.offset_x, &fallback.offset_x);
        select(&mut self.offset_y, &fallback.offset_y);

        self.fonts.append(&mut fallback.fonts.clone());
    }
}

/// Background of a component or the bar.
#[derive(Clone)]
pub enum Background {
    Image(DynamicImage),
    Color(Color),
}

impl Default for Background {
    fn default() -> Self {
        Background::Color(Color::new(0, 0, 0, 0))
    }
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
                    let path = Path::new(&text).canonicalize().map_err(D::Error::custom)?;
                    image::open(path)
                        .map(Background::Image)
                        .map_err(D::Error::custom)
                }
            }
            Err(err) => Err(err),
        }
    }
}

/// Distinct identification for a font.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize)]
pub struct Font {
    pub name: String,
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
///
/// These positions indicate where on the screen the bar should be displayed. The position `Top`
/// would indicate that the bar should be rendered at the top of the specified [`Monitor`].
///
/// [`Monitor`]: struct.Monitor.html
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Deserialize)]
pub enum Position {
    Top,
    Bottom,
}

impl Default for Position {
    fn default() -> Self {
        Position::Bottom
    }
}

/// RGBA color specified as four values from 0 to 255.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
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

        let radix_error =
            |_| String::from("hexadecimal color digits need to be within the range 0..=F");
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

    /// Convert the colors from whole numbers to floating point fractions.
    ///
    /// This converts the RGBA colors from the range 0..=255 to the range 0..1.0.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// use bar_config::bar::Bar;
    ///
    /// let input = Cursor::new(String::from("\
    ///      height: 30\n\
    ///      monitors:\n\
    ///      - { name: \"DVI-1\" }\n\
    ///      left:\n\
    ///      - { foreground: \"#FF00FF99\" }",
    /// ));
    ///
    /// let bar = Bar::load(input).unwrap();
    ///
    /// let foreground = bar.left()[0].settings().foreground.unwrap().as_f64();
    /// assert_eq!(foreground.0, 1.0);
    /// assert_eq!(foreground.1, 0.0);
    /// assert_eq!(foreground.2, 1.0);
    /// assert_eq!(foreground.3, 0.6);
    /// ```
    pub fn as_f64(self) -> (f64, f64, f64, f64) {
        (
            f64::from(self.r) / 255.,
            f64::from(self.g) / 255.,
            f64::from(self.b) / 255.,
            f64::from(self.a) / 255.,
        )
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
