use amethyst::ecs::{Component, VecStorage};
use std::convert::From;
pub struct ServoUrl {
    pub dirty: bool,
    pub url: String,
}

impl ServoUrl {
    pub fn goto(&mut self, url: String) {
        self.dirty = true;
        self.url = url.clone();
    }
}

impl From<String> for ServoUrl {
    fn from(string: String) -> Self {
        Self {
            dirty: true,
            url: string,
        }
    }
}

impl<'a> From<&'a str> for ServoUrl {
    fn from(string: &str) -> Self {
        Self {
            dirty: true,
            url: string.to_string(),
        }
    }
}

impl Component for ServoUrl {
    type Storage = VecStorage<ServoUrl>;
}
