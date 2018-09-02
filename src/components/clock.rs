use tokio::prelude::*;
use tokio::timer::Interval;

use std::time::{Duration, Instant};
use time;

use crate::components::{Component, ComponentID, ComponentSettings, ComponentStream};

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Clock {
    id: ComponentID,
    text: Option<String>,
    settings: Option<ComponentSettings>,
}

impl Component for Clock {
    fn text(&self) -> Option<String> {
        match time::now().strftime("%H:%m") {
            Ok(time) => Some(format!("{}", time)),
            Err(_) => None,
        }
    }

    fn settings(&self) -> Option<&ComponentSettings> {
        self.settings.as_ref()
    }

    fn stream(&self) -> ComponentStream {
        let id = self.id();
        let task =
            Interval::new(Instant::now(), Duration::from_millis(15000)).and_then(move |_| Ok(id));
        Box::new(task.map_err(|_| ()))
    }

    fn update(&mut self) -> bool {
        true
    }

    fn id(&self) -> ComponentID {
        self.id
    }
}
