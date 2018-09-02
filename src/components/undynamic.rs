use crate::components::{Component, ComponentID, ComponentSettings};

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Undynamic {
    id: ComponentID,
    text: Option<String>,
    settings: Option<ComponentSettings>,
}

impl Component for Undynamic {
    fn settings(&self) -> Option<&ComponentSettings> {
        self.settings.as_ref()
    }

    fn text(&self) -> Option<String> {
        self.text.clone()
    }

    fn id(&self) -> ComponentID {
        self.id
    }
}
