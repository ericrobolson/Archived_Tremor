pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a>;
}

pub type Index = u32;

pub fn wgpu_index_format() -> wgpu::IndexFormat {
    // Ensure this is the same type as index
    wgpu::IndexFormat::Uint32
}

pub fn indexed_quad(
    x: f32,
    y: f32,
    z: f32,
    width: f32,
    height: f32,
    quad_offset: Option<Index>,
) -> (Vec<[f32; 3]>, Vec<Index>) {
    let y = y - 1.0;

    let min_x = x;
    let max_x = x + width;
    let min_y = y;
    let max_y = y + height;

    // http://www.opengl-tutorial.org/intermediate-tutorials/tutorial-9-vbo-indexing/
    let verts = vec![
        // bot left
        [min_x, min_y, z],
        // bot right
        [max_x, min_y, z],
        // top left
        [min_x, max_y, z],
        // top right
        [max_x, max_y, z],
    ];

    let offset = match quad_offset {
        Some(o) => o,
        None => 0,
    };

    // Calculate the indexes. When using a buffer with a lot of 'quads', you'll need to update the offsets to point to the proper verts.
    let indexes: Vec<Index> = vec![
        0 + offset,
        1 + offset,
        2 + offset,
        2 + offset,
        1 + offset,
        3 + offset,
    ];

    (verts, indexes)
}

pub fn textured_indexed_quad(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    texture_region: Option<TextureRegion>,
    quad_offset: Option<Index>,
) -> (Vec<([f32; 3], [f32; 2])>, Vec<Index>) {
    let (verts, indexes) = indexed_quad(x, y, 0.0, width, height, quad_offset);

    let region = match texture_region {
        Some(tx) => tx,
        None => TextureRegion {
            min_x: 0.0,
            max_x: 1.0,
            min_y: 0.0,
            max_y: 1.0,
        },
    };

    let verts = verts
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let verts = *v;
            let tex_coords = {
                const BOT_LEFT_INDEX: usize = 0;
                const BOT_RIGHT_INDEX: usize = 1;
                const TOP_LEFT_INDEX: usize = 2;
                const TOP_RIGHT_INDEX: usize = 3;

                // bot left vert
                if i == BOT_LEFT_INDEX {
                    [region.min_x, region.max_y]
                }
                // bot right
                else if i == BOT_RIGHT_INDEX {
                    [region.max_x, region.max_y]
                }
                // top left
                else if i == TOP_LEFT_INDEX {
                    [region.min_x, region.min_y]
                }
                // top right
                else if i == TOP_RIGHT_INDEX {
                    [region.max_x, region.min_y]
                } else {
                    unimplemented!()
                }
            };

            (verts, tex_coords)
        })
        .collect();

    (verts, indexes)
}

pub struct TextureRegion {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}
