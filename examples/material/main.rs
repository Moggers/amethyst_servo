//! Displays spheres with physically based materials.

extern crate amethyst;
extern crate amethyst_servo;
extern crate genmesh;

use amethyst::assets::Loader;
use amethyst::core::cgmath::{Deg, InnerSpace, Matrix4, Vector3};
use amethyst::core::transform::GlobalTransform;
use amethyst::prelude::*;
use amethyst::renderer::*;
use genmesh::{MapToVertices, Triangulate, Vertex, Vertices};
use genmesh::generators::Plane;
use amethyst_servo::{ServoUiBundle, ServoUiTarget};

struct Example;

impl State for Example {
    fn on_start(&mut self, world: &mut World) {
        let mat_defaults = world.read_resource::<MaterialDefaults>().0.clone();
        let verts = gen_plane().into();
        println!("Load mesh");
        let (mesh, servo_target) = {
            let loader = world.read_resource::<Loader>();
            let meshes = &world.read_resource();
            let mesh: MeshHandle = loader.load_from_data(verts, (), meshes);

            let servo_target = world.read_resource::<ServoUiTarget>().handle.clone();

            (mesh, servo_target)
        };

        let pos = Matrix4::from_translation([0., 0., 0.].into());

        let mtl = Material {
            albedo: servo_target,
            ..mat_defaults.clone()
        };

        world
            .create_entity()
            .with(GlobalTransform(pos.into()))
            .with(mesh.clone())
            .with(mtl)
            .build();

        println!("Create lights");

        let light: Light = PointLight {
            center: [6.0, -6.0, -6.0].into(),
            intensity: 5.0,
            color: [1., 1., 1.].into(),
            ..PointLight::default()
        }.into();

        world.create_entity().with(light).build();

        println!("Put camera");

        let transform = Matrix4::from_translation([0.0, 0.0, 3.0].into());
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
        "{}/examples/material/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let config = DisplayConfig::load(&path);

    let resources = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawPbm::<PosNormTangTex>::new()),
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

fn gen_plane() -> Vec<PosNormTangTex> {
    Plane::new()
        .vertex(|vertex: Vertex| {
            let normal = Vector3::from([0., 0., -1.]);
            let up = Vector3::from([0.0, 1.0, 0.0]);
            let tangent = normal.cross(up).cross(normal);
            PosNormTangTex {
                position: vertex.pos,
                normal: normal.into(),
                tangent: tangent.into(),
                tex_coord: [vertex.pos[0], vertex.pos[1]],
            }
        })
        .triangulate()
        .vertices()
        .collect()
}