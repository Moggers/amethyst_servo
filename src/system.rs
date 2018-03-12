extern crate genmesh;

use amethyst::ecs::{RunningTime, System};
use amethyst::core::cgmath::{Matrix4, Vector3};
use amethyst::prelude::World;
use super::ServoHandle;
use amethyst::renderer::{Material, MaterialDefaults, MeshHandle, PosTex, Texture, TextureData,
                         TextureHandle, TextureMetadata};
use amethyst::core::transform::{GlobalTransform, Transform};
use amethyst::winit::Event;
use amethyst::shrev::{EventChannel, ReaderId};
use amethyst::shred::Fetch;
use amethyst::assets::{AssetStorage, Loader};

pub struct ServoUiSystem {
    reader_id: ReaderId<Event>,
    servo: ServoHandle,
}

impl ServoUiSystem {
    pub fn new(world: &mut World) -> Self {
        let target_handle: TextureHandle = {
            let tex_storage = world.read_resource();
            let loader = world.read_resource::<Loader>();
            let texture_data = TextureData::Rgba(
                [1., 0., 0., 1.],
                TextureMetadata {
                    sampler: None,
                    mip_levels: Some(1),
                    size: Some((100, 100)),
                    dynamic: false,
                    format: None,
                    channel: None,
                },
            );
            loader.load_from_data(texture_data, (), &tex_storage)
        };
        world.add_resource(ServoUiTarget::new(target_handle.clone()));
        Self {
            reader_id: world
                .write_resource::<EventChannel<Event>>()
                .register_reader(),
            servo: ServoHandle::start_servo(world),
        }
    }
}

impl<'a> System<'a> for ServoUiSystem {
    type SystemData = (
        Fetch<'a, EventChannel<Event>>,
        Fetch<'a, ServoUiTarget>,
        Fetch<'a, AssetStorage<Texture>>,
    );
    fn running_time(&self) -> RunningTime {
        RunningTime::Average
    }

    fn run(&mut self, (events, target, tex_storage): Self::SystemData) {
        match self.servo.window.has_target() {
            Ok(false) => match tex_storage.get(&target.handle) {
                Some(t) => {
                    self.servo.window.set_target(t);
                    match self.servo.window.setup_framebuffer() {
                        Ok(()) => println!("Setup framebuffer and render target"),
                        Err(e) => {
                            eprintln!("Failed to setup framebuffer and render target: {:?}", e)
                        }
                    }
                }
                None => {}
            },
            _ => {}
        }
        for event in events.read(&mut self.reader_id) {
            match event {
                &Event::Awakened => {
                    self.servo.update();
                }
                &Event::WindowEvent {
                    window_id: _window_id,
                    ref event,
                } => {
                    self.servo.forward_events(vec![event.clone()]);
                    // Send the resize through to servo
                }
                _ => {}
            }
        }
    }
}

pub struct ServoUiTarget {
    pub handle: TextureHandle,
}

impl ServoUiTarget {
    pub fn new(targ: TextureHandle) -> Self {
        Self { handle: targ }
    }
}
