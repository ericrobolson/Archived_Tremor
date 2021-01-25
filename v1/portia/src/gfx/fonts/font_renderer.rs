use rendering_ir::wgpu_helpers::{
    texture::{Image, TextureAtlas},
    vertex::Index,
};

use crate::{
    file_system::FileSystem,
    gfx::DeviceQueue,
    gui::{Color, ScreenPoint},
};
use rusttype::{point, Font, Scale};
use std::collections::HashMap;

use super::vertex::FontVert;

const MAX_CHARACTERS_IN_BUFFER: u32 = 4096;
const MAX_CHARACTERS_PER_ROW: u32 = 16;

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

pub struct FontRenderer {
    characters: HashMap<char, Character>,
    pub name: &'static str,
    font: rusttype::Font<'static>,
    scale: Scale,
    metrics: rusttype::VMetrics,
    offset: u32,
    next_location: (u32, u32),
    glyph_max_size: (u32, u32),

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

fn output_alpha(v: f32) -> u8 {
    let a = v * 255.0;

    return a as u8;
}

pub struct TextRenderCommand {
    pub font_size: f32,
    pub position: ScreenPoint,
    pub text: String,
    pub color: Color,
}

impl<'a> FontRenderer {
    pub fn new(
        font_name: &'static str,
        screen_size: (u32, u32),
        layout: &wgpu::BindGroupLayout,
        dq: &DeviceQueue,
    ) -> Self {
        let path = FileSystem::get_file(font_name);
        let data = std::fs::read(&path).unwrap();
        let font = Font::try_from_vec(data).unwrap_or_else(|| {
            panic!(format!("error constructing a Font from data at {:?}", path));
        });

        const font_size: u32 = 32;

        let scale = Scale::uniform(font_size as f32);
        let metrics = font.v_metrics(scale);
        let glyph_height = (metrics.ascent - metrics.descent).ceil() as u32;

        let atlas_size = (
            MAX_CHARACTERS_PER_ROW * font_size,
            MAX_CHARACTERS_PER_ROW * font_size,
        );

        let texture = match TextureAtlas::empty(
            dq.device,
            dq.queue,
            atlas_size.0,
            atlas_size.1,
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

            for _ in 0..MAX_CHARACTERS_IN_BUFFER {
                let (mut v, mut i) =
                    init_vert_indexes(0.0, 0.0, 0.0, 0.0, Color::white(), None, None);

                verts.append(&mut v);
                indexes.append(&mut i);
            }

            (verts, indexes)
        };

        use wgpu::util::DeviceExt;
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
            characters: HashMap::new(),
            font,
            name: font_name,
            scale,
            metrics,
            offset: font_size / 4,
            texture,
            tex_bind_group,
            next_location: (0, 0),
            glyph_max_size: (font_size, glyph_height),
            text_commands: Vec::with_capacity(500),
            //
            screen_size,
            vert_buffer,
            index_buffer,
            index_len: 0,
            index_offset: 0,
            vert_len: 0,
        };

        // populate with ASCII characters
        for i in 0..127 {
            let i = i as u8;
            let c = i as char;
            container.process_character(c, dq);
        }

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

        let mut verts = vec![];
        let mut indexes = vec![];
        let mut characters_in_buffer = 0;
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
                        if characters_in_buffer >= MAX_CHARACTERS_IN_BUFFER {
                            break;
                        }

                        characters_in_buffer += 1;

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

    // Given a character, ensure that it is created in the texture.
    fn process_character(&mut self, c: char, dq: &DeviceQueue) -> Character {
        match self.characters.get(&c) {
            Some(ch) => {
                return *ch;
            }
            None => {}
        }

        let glyph = self
            .font
            .glyph(c)
            .scaled(self.scale)
            .positioned(point(0.0, 0.0 + self.metrics.ascent));

        match glyph.pixel_bounding_box() {
            Some(bb) => {}
            None => {
                // Whitespace character
                let ch = Character {
                    c,
                    texture_location: (0., 0.),
                    texture_size: (0., 0.),
                    pixel_size: (self.glyph_max_size.0 / 4, 3),
                    is_whitespace: true,
                    texture_pixel_location: (0, 0),
                };

                self.characters.insert(c, ch);
                return ch;
            }
        }

        let height = self.glyph_max_size.1;
        let width = {
            let bb = glyph.pixel_bounding_box().unwrap();
            (bb.max.x - bb.min.x) as u32
        };
        let x_offset = self.offset / 2;

        let img = {
            let (img_width, img_height) = self.glyph_max_size;

            use image::{DynamicImage, GenericImage, Rgba};

            let mut img = DynamicImage::new_rgba8(img_width, img_height);
            let color = (255, 255, 255);

            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                // Draw the glyph into the image per-pixel by using the draw closure
                glyph.draw(|x, y, v| {
                    let px = x + x_offset;
                    let py = y + {
                        // Due to negative offsets, need to ensure that we don't go below 0.
                        let y = bounding_box.min.y;
                        if y < 0 {
                            0
                        } else {
                            bounding_box.min.y as u32
                        }
                    };

                    img.put_pixel(px, py, Rgba([color.0, color.1, color.2, output_alpha(v)]));
                });
            }

            img
        };

        // Write the character image to the texture atlas
        let (x, y) = self.next_location;
        {
            self.next_location = {
                let (glyph_w, glyph_h) = self.glyph_max_size;

                let (tex_w, _tex_h) = self.texture.size();
                let mut new_x = x + glyph_w;
                let mut new_y = y;
                if new_x >= tex_w {
                    new_x = 0;
                    new_y += glyph_h;
                }

                (new_x, new_y)
            };

            self.texture.write(x, y, &img, dq.queue);
        }

        // Create the character struct
        let (max_x, max_y) = self.texture.size();

        let texture_location = {
            let (x, y) = ((x + x_offset) as f32, y as f32);

            (x / max_x as f32, y / max_y as f32)
        };

        let texture_size = {
            let x = width + x_offset / 2;
            let y = self.glyph_max_size.1;

            let (max_w, max_h) = self.texture.size();

            ((x as f32 / max_w as f32), y as f32 / max_h as f32)
        };

        let ch = Character {
            c,
            texture_location,
            texture_size,
            pixel_size: (width, height),
            is_whitespace: false,
            texture_pixel_location: (x, y),
        };

        self.characters.insert(c, ch);

        ch
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
) -> (Vec<FontVert>, Vec<Index>) {
    let z = 0.0;
    let (verts, indexes) = textured_indexed_quad(x, y, w, h, texture_region, quad_offset);

    let color: [f32; 4] = color.into();

    let font_verts: Vec<FontVert> = verts
        .iter()
        .map(|(v, tex)| FontVert {
            position: [v[0], v[1]],
            texture_coords: *tex,
            color,
        })
        .collect();

    (font_verts, indexes)
}

fn indexed_quad(
    x: f32,
    y: f32,
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
    let z = 0.0;
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

fn textured_indexed_quad(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    texture_region: Option<TextureRegion>,
    quad_offset: Option<Index>,
) -> (Vec<([f32; 3], [f32; 2])>, Vec<Index>) {
    let (verts, indexes) = indexed_quad(x, y, width, height, quad_offset);

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

struct TextureRegion {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}
