extern crate amethyst;
extern crate amethyst_servo;
extern crate genmesh;
extern crate hibitset;

use amethyst::core::cgmath::{Deg, Matrix4};
use amethyst::core::transform::GlobalTransform;
use amethyst::prelude::*;
use amethyst::renderer::*;
use amethyst_servo::{ServoBlit, ServoPass, ServoSize, ServoUiBundle, ServoUrl};

struct Example;

impl State for Example {
    fn on_start(&mut self, world: &mut World) {
        println!("Create servo");
        let material = { world.read_resource::<MaterialDefaults>().0.clone() };
        world
            .create_entity()
            .with::<ServoUrl>(
                format!(
                    "file://{}/examples/assets/test.html",
                    env!("CARGO_MANIFEST_DIR")
                ).into(),
            )
            .with::<ServoSize>((1024, 1024).into())
            .with(ServoBlit {})
            .with(material)
            .build();

        println!("Create lights");
        let light: Light = PointLight {
            center: [6.0, -6.0, -6.0].into(),
            intensity: 5.0,
            color: [1., 1., 1.].into(),
            ..PointLight::default()
        }.into();
        world.create_entity().with(light).build();

        println!("Create camera");
        let transform = Matrix4::from_translation([0.0, 0.0, 2.0].into());
        world
            .create_entity()
            .with(Camera::from(Projection::perspective(1.3, Deg(60.0))))
            .with(GlobalTransform(transform.into()))
            .build();
    }

    fn handle_event(&mut self, _: &mut World, event: Event) -> Trans {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => Trans::Quit,
                _ => Trans::None,
            },
            _ => Trans::None,
        }
    }
}

fn run() -> Result<(), amethyst::Error> {
    let path = format!(
        "{}/examples/blit/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let config = DisplayConfig::load(&path);

    let resources = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawPbm::<PosNormTangTex>::new())
            .with_pass(ServoPass::new()),
    );
    let mut game = Application::build(&resources, Example)?
        .with_bundle(RenderBundle::new(pipe, Some(config)))?
        .with_bundle(ServoUiBundle {})?
        .build()?;
    game.run();
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}
