pub use amethyst::ecs::{Component, VecStorage};
pub struct ServoSize {
    pub width: u32,
    pub height: u32,
    pub dirty: bool,
}

impl Component for ServoSize {
    type Storage = VecStorage<ServoSize>;
}

impl From<(u32, u32)> for ServoSize {
    fn from(dim: (u32, u32)) -> Self {
        Self {
            width: dim.0,
            height: dim.1,
            dirty: true,
        }
    }
}

impl ServoSize {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width: width,
            height: height,
            dirty: true,
        }
    }
}
