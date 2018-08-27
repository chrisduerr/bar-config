#[cfg(all(feature = "json-fmt", not(feature = "toml-fmt")))]
use serde_json;
#[cfg(not(any(feature = "toml-fmt", feature = "json-fmt")))]
use serde_yaml;
#[cfg(all(feature = "toml-fmt", not(feature = "json-fmt")))]
use toml;

use serde::de::{Deserializer, Error};
use serde::ser::Serialize;
use serde::{Deserialize, Serializer};

use std::path::{Path, PathBuf};

/// Root element of the bar
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Bar {
    /// General bar configuration settings
    pub height: u8,
    pub position: Option<Position>,
    pub background: Option<Background>,
    #[serde(deserialize_with = "deserialize_monitors", skip_serializing_if = "Vec::is_empty")]
    pub monitors: Vec<Monitor>,

    /// Default fallback values for components
    pub defaults: Option<ComponentSettings>,

    /// Component containers
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub left: Vec<Component>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub center: Vec<Component>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub right: Vec<Component>,
}

// Require at least one monitor
fn deserialize_monitors<'a, D>(deserializer: D) -> ::std::result::Result<Vec<Monitor>, D::Error>
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

/// A single component/block/module in the bar
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Component {
    /// Name used to identify which component should be loaded
    pub name: Option<String>,

    // Text which will be displayed inside the component
    pub text: Option<String>,

    /// Options available for all components
    pub settings: Option<ComponentSettings>,

    /// Extra options which are passed to the component
    #[cfg(not(any(feature = "toml-fmt", feature = "json-fmt")))]
    pub component_options: Option<serde_yaml::Value>,
    #[cfg(all(feature = "json-fmt", not(feature = "toml-fmt")))]
    pub component_options: Option<serde_json::Value>,
    #[cfg(all(feature = "toml-fmt", not(feature = "json-fmt")))]
    pub component_options: Option<toml::Value>,
}

/// Default options available for every component
#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
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

/// Background of a component or the bar
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Background {
    Image(PathBuf),
    Color(Color),
}

impl<'de> Deserialize<'de> for Background {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Background, D::Error>
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

impl Serialize for Background {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let text = match *self {
            Background::Image(ref path) => path.to_string_lossy().into_owned(),
            Background::Color(ref color) => color.to_string(),
        };
        serializer.serialize_str(&text)
    }
}

/// Distinct identification for a font
#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Font {
    pub description: String,
    pub size: u8,
}

/// Distinct identification for a monitor
#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Monitor {
    pub name: String,
    #[serde(default)]
    pub fallback_names: Vec<String>,
}

/// Border separating the bar from the rest of the WM
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Border {
    pub height: u8,
    pub color: Color,
}

/// Available positions for the bar
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum Position {
    Top,
    Bottom,
}

/// RGBA color specified as four values from 0 to 255
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

        for c in (&string.to_uppercase()[1..]).chars() {
            let char_code = c as u8;
            // Make sure the char lies within the range 0..9 or A..F
            if char_code < 48 || char_code > 70 || (char_code > 57 && char_code < 65) {
                return Err(String::from(
                    "hexadecimal color digits need to be within the range 0..=F",
                ));
            }
        }

        let r = u8::from_str_radix(&string[1..3], 16).unwrap();
        let g = u8::from_str_radix(&string[3..5], 16).unwrap();
        let b = u8::from_str_radix(&string[5..7], 16).unwrap();
        let a = if string.len() == 9 {
            u8::from_str_radix(&string[7..9], 16).unwrap()
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
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer) {
            Ok(color_string) => Color::from_str(&color_string).map_err(D::Error::custom),
            Err(err) => Err(err),
        }
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod test {
    #[cfg(all(feature = "json-fmt", not(feature = "toml-fmt")))]
    use serde_json as serde_fmt;
    #[cfg(not(any(feature = "toml-fmt", feature = "json-fmt")))]
    use serde_yaml as serde_fmt;
    #[cfg(all(feature = "toml-fmt", not(feature = "json-fmt")))]
    use toml as serde_fmt;

    #[cfg(all(feature = "json-fmt", not(feature = "toml-fmt")))]
    const TEST_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/json/");
    #[cfg(not(any(feature = "toml-fmt", feature = "json-fmt")))]
    const TEST_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/yaml/");
    #[cfg(all(feature = "toml-fmt", not(feature = "json-fmt")))]
    const TEST_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/toml/");

    use std::fs::File;
    use std::io::Read;
    use std::path::Path;

    use config::{Background, Bar};

    #[test]
    fn minimal_config() {
        let path = [TEST_DIR, "minimal_config"].concat();

        let mut input_text = String::new();
        let mut file = File::open(path).unwrap();
        file.read_to_string(&mut input_text).unwrap();

        let config: Bar = serde_fmt::from_str(&input_text).unwrap();

        assert_eq!(config.height, 30);
        assert_eq!(config.monitors.len(), 1);
        assert_eq!(config.monitors[0].name, "DVI-1");
        assert!(config.monitors[0].fallback_names.is_empty());
    }

    #[test]
    fn full_config() {
        let path = [TEST_DIR, "full_config"].concat();

        let mut input_text = String::new();
        let mut file = File::open(path).unwrap();
        file.read_to_string(&mut input_text).unwrap();

        let config: Bar = serde_fmt::from_str(&input_text).unwrap();
        let output_text = serde_fmt::to_string(&config).unwrap();

        assert_eq!(input_text, output_text + "\n");
    }

    #[test]
    fn path_in_config() {
        let path = [TEST_DIR, "path_in_config"].concat();
        let expected = Background::Image(Path::new("/home").canonicalize().unwrap());

        let mut input_text = String::new();
        let mut file = File::open(path).unwrap();
        file.read_to_string(&mut input_text).unwrap();

        let config: Bar = serde_fmt::from_str(&input_text).unwrap();

        assert_eq!(config.background.unwrap(), expected);
    }
}
