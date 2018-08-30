#[cfg(all(feature = "json-fmt", not(feature = "toml-fmt")))]
use serde_json as serde_fmt;
#[cfg(not(any(feature = "toml-fmt", feature = "json-fmt")))]
use serde_yaml as serde_fmt;
#[cfg(all(feature = "toml-fmt", not(feature = "json-fmt")))]
use toml as serde_fmt;

use serde::de::{Deserializer, Error};
use serde::Deserialize;

mod undynamic;

use crate::config::ComponentSettings;
use crate::components::undynamic::Undynamic;

pub trait Component: ::std::fmt::Debug {
    fn text(&self) -> Option<&String> {
        None
    }
    fn settings(&self) -> Option<&ComponentSettings> {
        None
    }
}

impl<'de> Deserialize<'de> for Box<dyn Component> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = serde_fmt::Value::deserialize(deserializer)?;

        let name = match val.get("name") {
            Some(serde_fmt::Value::String(name)) => Some(name.to_owned()),
            _ => {
                return Ok(Box::new(
                    Undynamic::deserialize(val).map_err(D::Error::custom)?,
                ))
            }
        };

        match name {
            _ => Ok(Box::new(
                Undynamic::deserialize(val).map_err(D::Error::custom)?,
            )),
        }
    }
}
