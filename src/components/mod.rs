#[cfg(all(feature = "json-fmt", not(feature = "toml-fmt")))]
use serde_json as serde_fmt;
#[cfg(not(any(feature = "toml-fmt", feature = "json-fmt")))]
use serde_yaml as serde_fmt;
#[cfg(all(feature = "toml-fmt", not(feature = "json-fmt")))]
use toml as serde_fmt;

mod clock;
mod undynamic;

use serde::de::{Deserializer, Error};
use serde::Deserialize;
use tokio::prelude::stream::{self, Stream};

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::components::clock::Clock;
use crate::components::undynamic::Undynamic;
use crate::config::ComponentSettings;

static COMPONENT_INDEX: AtomicUsize = AtomicUsize::new(0);

pub(crate) type ComponentStream = Box<Stream<Item = ComponentID, Error = ()> + Send>;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub struct ComponentID(usize);

impl<'de> Deserialize<'de> for ComponentID {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(ComponentID(COMPONENT_INDEX.fetch_add(1, Ordering::Relaxed)))
    }
}

pub trait Component: ::std::fmt::Debug + Send {
    fn text(&self) -> Option<String> {
        None
    }

    fn settings(&self) -> Option<&ComponentSettings> {
        None
    }

    fn stream(&self) -> ComponentStream {
        Box::new(stream::empty())
    }

    fn update(&mut self) -> bool {
        false
    }

    fn id(&self) -> ComponentID;
}

impl<'de> Deserialize<'de> for Box<dyn Component> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = serde_fmt::Value::deserialize(deserializer)?;

        let name = match val.get("name") {
            Some(serde_fmt::Value::String(name)) => name.to_owned(),
            _ => {
                return Ok(Box::new(
                    Undynamic::deserialize(val).map_err(D::Error::custom)?,
                ));
            }
        };

        Ok(match name.as_str() {
            "clock" => Box::new(Clock::deserialize(val).map_err(D::Error::custom)?),
            _ => Box::new(Undynamic::deserialize(val).map_err(D::Error::custom)?),
        })
    }
}
