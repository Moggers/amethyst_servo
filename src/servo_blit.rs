use amethyst::ecs::{Component, VecStorage};
pub struct ServoBlit {}

impl Component for ServoBlit {
    type Storage = VecStorage<ServoBlit>;
}
