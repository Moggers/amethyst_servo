use amethyst::core::bundle::{ECSBundle, Result};
use amethyst::ecs::DispatcherBuilder;
use amethyst::prelude::World;
use super::{ServoHandle, ServoSize, ServoUiSystem, ServoUrl};

pub struct ServoUiBundle;
impl<'a, 'b> ECSBundle<'a, 'b> for ServoUiBundle {
    fn build(
        self,
        world: &mut World,
        dispatcher: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        world.register::<ServoUrl>();
        world.register::<ServoHandle>();
        world.register::<ServoSize>();
        Ok(dispatcher.add_thread_local(ServoUiSystem::new(world)))
    }
}
