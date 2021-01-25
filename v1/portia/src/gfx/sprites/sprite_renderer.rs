use rendering_ir::wgpu_helpers::{
    texture::{Image, TextureAtlas},
    vertex::{textured_indexed_quad, Index, TextureRegion},
};

use wgpu::util::DeviceExt;

use crate::{
    file_system::FileSystem,
    gfx::DeviceQueue,
    gui::{Color, ScreenPoint},
};

use super::vertex::SpriteVert;

const MAX_INSTANCES_IN_BUFFER: u32 = 4096;
const TEXTURE_ATLAS_SIZE: (u32, u32) = (2056, 2056);

struct BindGroups;
impl BindGroups {
    const TEXTURE: u32 = 0;
}

#[derive(Copy, Clone, Debug)]
pub struct Character {
    c: char,
    pub pixel_size: (u32, u32),
    pub texture_location: (f32, f32),
    pub texture_size: (f32, f32),
    pub is_whitespace: bool,
    pub texture_pixel_location: (u32, u32),
}

impl Character {
    fn texture_region(&self) -> TextureRegion {
        let (x, y) = self.texture_location;
        let (w, h) = self.texture_size;

        TextureRegion {
            min_x: x,
            max_x: x + w,
            min_y: y,
            max_y: y + h,
        }
    }
}

pub struct SpriteRenderer {
    pub name: &'static str,
    text_commands: Vec<TextRenderCommand>,

    // WGPU stuff
    screen_size: (u32, u32),
    tex_bind_group: wgpu::BindGroup,
    texture: TextureAtlas,
    vert_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_len: u32,
    index_offset: u32,
    vert_len: u32,
}

pub struct TextRenderCommand {
    pub font_size: f32,
    pub position: ScreenPoint,
    pub text: String,
    pub color: Color,
}

impl<'a> SpriteRenderer {
    pub fn new(
        sprite_name: &'static str,
        screen_size: (u32, u32),
        layout: &wgpu::BindGroupLayout,
        dq: &DeviceQueue,
    ) -> Self {
        let texture = match TextureAtlas::empty(
            dq.device,
            dq.queue,
            TEXTURE_ATLAS_SIZE.0,
            TEXTURE_ATLAS_SIZE.1,
            None,
            false,
            false,
        ) {
            Ok(texture) => texture,
            Err(e) => {
                panic!("{:?}", e);
            }
        };

        let tex_bind_group = dq.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler()),
                },
            ],
            label: Some("character texture"),
        });

        // Prepopulate the buffers to a max range
        let (verts, indexes) = {
            let mut verts = vec![];
            let mut indexes = vec![];

            for _ in 0..MAX_INSTANCES_IN_BUFFER {
                let (mut v, mut i) =
                    init_vert_indexes(0.0, 0.0, 0.0, 0.0, Color::white(), None, None);

                verts.append(&mut v);
                indexes.append(&mut i);
            }

            (verts, indexes)
        };

        let vert_buffer = dq
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&verts),
                usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            });

        let index_buffer = dq
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&indexes),
                usage: wgpu::BufferUsage::INDEX | wgpu::BufferUsage::COPY_DST,
            });

        let mut container = Self {
            name: sprite_name,
            texture,
            tex_bind_group,
            text_commands: Vec::with_capacity(500),
            //
            screen_size,
            vert_buffer,
            index_buffer,
            index_len: 0,
            index_offset: 0,
            vert_len: 0,
        };

        container
    }

    pub fn resize(&mut self, screen_width: u32, screen_height: u32) {
        self.screen_size = (screen_width, screen_height);
    }

    pub fn render<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>) {
        let tex_bind_group = &self.tex_bind_group;
        let vert_buf = &self.vert_buffer;
        let index_buf = &self.index_buffer;
        let index_len = self.index_len;

        render_pass.set_bind_group(BindGroups::TEXTURE, tex_bind_group, &[]);
        render_pass.set_vertex_buffer(0, vert_buf.slice(..));
        render_pass.set_index_buffer(index_buf.slice(..));
        render_pass.draw_indexed(0..index_len, 0, 0..1);
    }

    pub fn reset(&mut self) {
        self.vert_len = 0;
        self.index_len = 0;
        self.index_offset = 0;
        self.text_commands.clear();
    }

    pub fn add_command(&mut self, text_command: TextRenderCommand) {
        self.text_commands.push(text_command);
    }

    /// Position is the placement of the text on the screen.
    pub fn execute(&mut self, dq: &DeviceQueue) {
        // Ensure the character is created for the font

        let mut verts: Vec<SpriteVert> = vec![];
        let mut indexes: Vec<Index> = vec![];
        let mut sprites_in_buffer = 0;
        loop {
            match self.text_commands.pop() {
                Some(text_command) => {
                    let position = text_command.position;
                    let font_size = text_command.font_size;
                    let mut x = position.x;
                    let mut y = position.y;

                    let pixel_w_ratio = font_size / self.screen_size.0 as f32;
                    let pixel_h_ratio = font_size / self.screen_size.1 as f32;

                    // Add verts + indexes to buffer for each character after processing it.
                    for c in text_command.text.chars() {
                        // Ensure we don't overflow the buffers
                        if sprites_in_buffer >= MAX_INSTANCES_IN_BUFFER {
                            break;
                        }

                        sprites_in_buffer += 1;
                        // TODO: write verts
                        /*
                        let character = self.process_character(c, dq);

                        // Offset for character positions and apply to next
                        let w = character.pixel_size.0 as f32 * pixel_w_ratio;
                        let h = character.pixel_size.1 as f32 * pixel_h_ratio;
                        let (mut v, mut i) = init_vert_indexes(
                            x,
                            y + pixel_h_ratio * self.glyph_max_size.0 as f32,
                            w,
                            h,
                            text_command.color,
                            Some(character.texture_region()),
                            Some(self.index_offset),
                        );

                        x += w;
                        self.index_offset += v.len() as Index;

                        verts.append(&mut v);
                        indexes.append(&mut i);
                        */
                    }
                }
                None => {
                    break;
                }
            }
        }

        self.vert_len += verts.len() as u32;
        self.index_len += indexes.len() as u32;

        // Write to buffers
        dq.queue
            .write_buffer(&self.vert_buffer, 0, bytemuck::cast_slice(&verts));

        dq.queue
            .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indexes));
    }
}

fn init_vert_indexes(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: Color,
    texture_region: Option<TextureRegion>,
    quad_offset: Option<Index>,
) -> (Vec<SpriteVert>, Vec<Index>) {
    let z = 0.0;
    let (verts, indexes) = textured_indexed_quad(x, y, w, h, texture_region, quad_offset);

    let color: [f32; 4] = color.into();

    let font_verts: Vec<SpriteVert> = verts
        .iter()
        .map(|(v, tex)| SpriteVert {
            position: [v[0], v[1]],
            texture_coords: *tex,
            color,
        })
        .collect();

    (font_verts, indexes)
}
