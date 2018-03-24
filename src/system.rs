extern crate genmesh;

use std::ops::Deref;
use amethyst::ecs::{Entities, Join, RunningTime, System, WriteStorage};
use glutin::GlWindow;
use std::sync::Arc;
use amethyst::prelude::World;
use super::{ServoHandle, ServoSize, ServoUrl};
use amethyst::renderer::{Material, Texture, TextureData, TextureMetadata};
use amethyst::winit::{Event, EventsLoopProxy};
use amethyst::shrev::{EventChannel, ReaderId};
use amethyst::shred::Fetch;
use amethyst::assets::{AssetStorage, Loader};
use hibitset::BitSetNot;

pub struct ServoUiSystem {
    reader_id: ReaderId<Event>,
}

impl ServoUiSystem {
    pub fn new(world: &mut World) -> Self {
        Self {
            reader_id: world
                .write_resource::<EventChannel<Event>>()
                .register_reader(),
        }
    }
}

impl<'a> System<'a> for ServoUiSystem {
    type SystemData = (
        WriteStorage<'a, ServoHandle>,
        WriteStorage<'a, ServoUrl>,
        WriteStorage<'a, ServoSize>,
        WriteStorage<'a, Material>,
        Entities<'a>,
        Fetch<'a, EventChannel<Event>>,
        Fetch<'a, AssetStorage<Texture>>,
        Fetch<'a, Arc<GlWindow>>,
        Fetch<'a, EventsLoopProxy>,
        Fetch<'a, Loader>,
    );
    fn running_time(&self) -> RunningTime {
        RunningTime::Average
    }

    fn run(
        &mut self,
        (
            mut servo_handles,
            mut urls,
            mut sizes,
            mut materials,
            entities,
            events,
            tex_storage,
            gl_window,
            event_proxy,
            loader,
        ): Self::SystemData,
    ) {
        // INIT ROUTINE
        for (entity, url, _) in (
            &*entities,
            &mut urls,
            &BitSetNot(servo_handles.open().0.clone()),
        ).join()
        {
            servo_handles.insert(
                entity,
                ServoHandle::start_servo(gl_window.deref(), event_proxy.deref(), &url.url),
            );
        }

        // TEXTURE ROUTINE
        for (size, servo, material) in (&mut sizes, &servo_handles, &mut materials).join() {
            if size.dirty == true {
                servo.window.set_dimensions(size.width, size.height);
                let texture_data = TextureData::Rgba(
                    [1., 1., 1., 0.],
                    TextureMetadata {
                        sampler: None,
                        mip_levels: Some(1),
                        size: Some((size.width as u16, size.height as u16)),
                        dynamic: false,
                        format: None,
                        channel: None,
                    },
                );
                let tex_handle = loader.load_from_data(texture_data, (), &tex_storage);
                size.dirty = false;
                material.albedo = tex_handle;
                if let Err(e) = servo.window.remove_target() {
                    panic!("Failed to remove old target: {:?}", e);
                }
            }
        }

        // EVENT ROUTINE
        for (handle, material, url) in (&mut servo_handles, &mut materials, &mut urls).join() {
            match handle.window.has_target() {
                Ok(false) => match tex_storage.get(&material.albedo) {
                    Some(t) => match handle.window.setup_framebuffer(t) {
                        Ok(()) => println!("Setup framebuffer and render target"),
                        Err(e) => {
                            eprintln!("Failed to setup framebuffer and render target: {:?}", e)
                        }
                    },
                    None => {}
                },
                _ => {}
            }
            if url.dirty == true {
                if let Err(e) = handle.navigate(&url.url) {
                    eprintln!("Failed navigation: {}", e);
                } else {
                    url.dirty = false;
                }
            }
        }
        for event in events.read(&mut self.reader_id) {
            for (handle,) in (&mut servo_handles,).join() {
                match event {
                    &Event::Awakened => {
                        handle.update();
                    }
                    _ => {}
                }
            }
        }
    }
}
