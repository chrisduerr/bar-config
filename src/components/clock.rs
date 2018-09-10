#[cfg(all(feature = "json-fmt", not(feature = "toml-fmt")))]
use serde_json as serde_fmt;
#[cfg(not(any(feature = "toml-fmt", feature = "json-fmt")))]
use serde_yaml as serde_fmt;
#[cfg(all(feature = "toml-fmt", not(feature = "json-fmt")))]
use toml as serde_fmt;

use serde::de::Deserialize;
use tokio::prelude::*;
use tokio::timer::Interval;

use std::time::{Duration, Instant};
use time;

use crate::components::{
    Component, ComponentID, ComponentSettings, ComponentStream, ComponentTrait,
};

const DEFAULT_INTERVAL_MILLIS: u64 = 15000;

pub struct Clock {
    id: ComponentID,
    settings: ComponentSettings,
    extra: Extra,
}

#[derive(Deserialize)]
struct Extra {
    interval: Option<u64>,
}

impl ComponentTrait for Clock {
    fn text(&self) -> String {
        match time::now().strftime("%H:%M") {
            Ok(time) => format!("{}", time),
            _ => String::new(),
        }
    }

    fn settings(&self) -> &ComponentSettings {
        &self.settings
    }

    fn stream(&self) -> ComponentStream {
        let id = self.id();
        let dur = Duration::from_millis(self.extra.interval.unwrap_or(DEFAULT_INTERVAL_MILLIS));
        let task = Interval::new(Instant::now() + dur, dur).and_then(move |_| Ok(id));
        Box::new(task.map_err(|_| ()))
    }

    fn update(&mut self) -> bool {
        true
    }

    fn id(&self) -> ComponentID {
        self.id
    }
}

impl Clock {
    pub(crate) fn create(settings: ComponentSettings, extra: serde_fmt::Value) -> Component {
        Component(Box::new(Self {
            settings,
            id: ComponentID::default(),
            extra: Extra::deserialize(extra).unwrap(),
        }))
    }
}
