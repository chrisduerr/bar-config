#[cfg(all(feature = "json-fmt", not(feature = "toml-fmt")))]
use serde_json as serde_fmt;
#[cfg(not(any(feature = "toml-fmt", feature = "json-fmt")))]
use serde_yaml as serde_fmt;
#[cfg(all(feature = "toml-fmt", not(feature = "json-fmt")))]
use toml as serde_fmt;

use crate::components::{Component, ComponentID, ComponentSettings, ComponentTrait};

use serde::de::Deserialize;

pub struct Undynamic {
    id: ComponentID,
    settings: ComponentSettings,
    extra: Extra,
}

#[derive(Deserialize)]
struct Extra {
    #[serde(default)]
    text: String,
}

impl ComponentTrait for Undynamic {
    fn text(&self) -> String {
        self.extra.text.clone()
    }

    fn settings(&self) -> &ComponentSettings {
        &self.settings
    }

    fn id(&self) -> ComponentID {
        self.id
    }
}

impl Undynamic {
    pub(crate) fn create(settings: ComponentSettings, extra: serde_fmt::Value) -> Component {
        Component(Box::new(Self {
            settings,
            id: ComponentID::default(),
            extra: Extra::deserialize(extra).unwrap(),
        }))
    }
}
