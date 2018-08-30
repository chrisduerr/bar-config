use crate::components::Component;
use crate::config::ComponentSettings;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Undynamic {
    text: Option<String>,
    settings: Option<ComponentSettings>,
}

impl Component for Undynamic {
    fn text(&self) -> Option<&String> {
        self.text.as_ref()
    }

    fn settings(&self) -> Option<&ComponentSettings> {
        self.settings.as_ref()
    }
}
