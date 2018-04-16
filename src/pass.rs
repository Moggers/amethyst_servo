use amethyst::renderer::pipe::pass::{Pass, PassData};
use amethyst::renderer::{DepthMode, Effect, Encoder, Factory, Material, Mesh, NewEffect, PosTex,
                         Texture, VertexFormat};
use amethyst::renderer::error::Result as RendererResult;
use amethyst::ecs::{Fetch, Join, ReadStorage};
use amethyst::assets::AssetStorage;
use gfx_core::pso::ElemStride;
use gfx_core::state::ColorMask;
use draw_state::preset::blend;
use super::ServoBlit;

pub struct ServoPass {
    mesh: Option<Mesh>,
}

impl ServoPass {
    pub fn new() -> Self {
        Self { mesh: None }
    }
}

type ServoPassData<'a> = (
    ReadStorage<'a, ServoBlit>,
    ReadStorage<'a, Material>,
    Fetch<'a, AssetStorage<Texture>>,
);

impl<'a> PassData<'a> for ServoPass {
    type Data = ServoPassData<'a>;
}

const VERT_SRC: &[u8] = include_bytes!("shaders/shader.vert");
const FRAG_SRC: &[u8] = include_bytes!("shaders/shader.frag");

impl Pass for ServoPass {
    fn compile(&mut self, mut effect: NewEffect) -> RendererResult<Effect> {
        let data = vec![
            PosTex {
                position: [0., 1., 0.],
                tex_coord: [0., 1.],
            },
            PosTex {
                position: [1., 1., 0.],
                tex_coord: [1., 1.],
            },
            PosTex {
                position: [1., 0., 0.],
                tex_coord: [1., 0.],
            },
            PosTex {
                position: [0., 1., 0.],
                tex_coord: [0., 1.],
            },
            PosTex {
                position: [1., 0., 0.],
                tex_coord: [1., 0.],
            },
            PosTex {
                position: [0., 0., 0.],
                tex_coord: [0., 0.],
            },
        ];
        self.mesh = Some(Mesh::build(data).build(&mut effect.factory)?);
        effect
            .simple(VERT_SRC, FRAG_SRC)
            .with_raw_vertex_buffer(PosTex::ATTRIBUTES, PosTex::size() as ElemStride, 0)
            .with_texture("albedo")
            .with_blended_output("color", ColorMask::all(), blend::ALPHA, None)
            .build()
    }
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        effect: &mut Effect,
        _factory: Factory,
        (blits, materials, tex_storage): ServoPassData,
    ) {
        let mesh = self.mesh.as_ref().unwrap();

        match mesh.buffer(PosTex::ATTRIBUTES) {
            Some(vbuf) => effect.data.vertex_bufs.push(vbuf.clone()),
            None => return,
        };
        for (material, _) in (&materials, &blits).join() {
            if let Some(image) = tex_storage.get(&material.albedo) {
                effect.data.textures.push(image.view().clone());
                effect.data.samplers.push(image.sampler().clone());
                effect.draw(mesh.slice(), encoder);
                effect.data.textures.clear();
                effect.data.samplers.clear();
            }
        }
    }
}
