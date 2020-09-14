use super::*;
use super::{texture::Texture, vertices::Vertices};

extern crate gl;

/// GFX representation of a sprite.
#[derive(Clone)]
pub struct Sprite {
    frame_index: u32,
    frame_count: u32,
    frame_height: u32,
    frame_width: u32,

    texture_id: gl::types::GLuint,
    texture_height: u32,
    texture_width: u32,

    scale_x: f32,
    scale_y: f32,
    position_x: f32,
    position_y: f32,
    r: f32,
    g: f32,
    b: f32,

    visible: bool,
}

impl Sprite {
    pub fn new(
        texture: &Texture,
        frame_count: Option<u32>,
        frame_index: Option<u32>,
        frame_height: Option<u32>,
        frame_width: Option<u32>,
    ) -> Self {
        let frame_height = frame_height.unwrap_or(texture.height());
        let frame_width = frame_width.unwrap_or(texture.width());

        Self {
            frame_index: frame_index.unwrap_or(0),
            frame_count: frame_count.unwrap_or(0),
            frame_height: frame_height,
            frame_width: frame_width,
            texture_id: texture.id(),
            texture_height: texture.height(),
            texture_width: texture.width(),
            scale_x: 1.0,
            scale_y: 1.0,
            position_x: 0.0,
            position_y: 0.0,
            r: 1.0,
            g: 1.0,
            b: 1.0,
            visible: true,
        }
    }

    pub fn increment_frame(&mut self) {
        self.frame_index += 1;
        if self.frame_count <= self.frame_index {
            self.frame_index = 0;
        }
    }

    pub fn texture_id(&self) -> gl::types::GLuint {
        self.texture_id
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position_x = x;
        self.position_y = y;
    }

    pub fn set_visible(&mut self, visible: bool) {
        unimplemented!("TODO: Visiblility not implemented!");
        self.visible = visible;
    }

    /// Convert the sprite into vertices that can be rendered
    pub fn into_verts(&self, resolution: Resolution) -> Vertices {
        if !self.visible {
            //TODO: implement visiblility
        }

        // Get the resolution/height of the screen as floats
        let res_w = resolution.width as f32;
        let res_h = resolution.height as f32;

        // We orient the sprite so the center of the frame is where the position is
        let half_width = (self.frame_width as f32 * self.scale_x) / 2.0;
        let half_height = (self.frame_height as f32 * self.scale_y) / 2.0;

        let mut start_x = self.position_x - half_width;
        let mut start_y = self.position_y - half_height;

        let mut end_x = self.position_x + half_width;
        let mut end_y = self.position_y + half_height;

        // This piece converts it from pixels to NDC
        {
            start_x /= res_w;
            end_x /= res_w;

            start_y /= res_h;
            end_y /= res_h;
        }

        // Placeholder for zindex
        let z = 0.0;

        // Initialize the colors
        let r = self.r;
        let g = self.g;
        let b = self.b;

        // Initialize the vertices
        const VERT_POSITION_LEN: usize = 3;
        const COLOR_POSITION_LEN: usize = 3;

        const TEX_COORDS_LEN: usize = 2;

        // Map the texture to the frame
        let frame_height_normalized = (self.frame_height as f32) / (self.texture_height as f32);
        let frame_width_normalized = (self.frame_width as f32) / (self.texture_width as f32);

        let (frame_x_index, frame_y_index) = self.calculate_xy_indexes_from_frame_index();

        let tex_min_x =
            0.0 * frame_width_normalized + frame_width_normalized * frame_x_index as f32;
        let tex_max_x =
            1.0 * frame_width_normalized + frame_width_normalized * frame_x_index as f32;
        let tex_min_y =
            1.0 * frame_height_normalized + frame_height_normalized * frame_y_index as f32;
        let tex_max_y =
            0.0 * frame_height_normalized + frame_height_normalized * frame_y_index as f32;

        // Generate the verts
        let quad_verts: Vec<f32> = vec![
            // Positions // Colors
            // Triangle 1
            start_x, start_y, z, r, g, b, tex_min_x, tex_min_y, // bottom left
            start_x, end_y, z, r, g, b, tex_min_x, tex_max_y, // top left
            end_x, end_y, z, r, g, b, tex_max_x, tex_max_y, // top right
            // Triangle 2
            end_x, end_y, z, r, g, b, tex_max_x, tex_max_y, // top right
            end_x, start_y, z, r, g, b, tex_max_x, tex_min_y, // bottom right
            start_x, start_y, z, r, g, b, tex_min_x, tex_min_y, // bottom left
        ];

        let verts = quad_verts;

        let vertices = Vertices::new(
            verts,
            vec![VERT_POSITION_LEN, COLOR_POSITION_LEN, TEX_COORDS_LEN],
        );

        return vertices;
    }

    fn calculate_xy_indexes_from_frame_index(&self) -> (u32, u32) {
        if self.frame_count == 0 {
            return (0, 0);
        }

        let x_frames = self.texture_width / self.frame_width;
        let y_frames = self.texture_height / self.frame_height;

        (self.frame_index % x_frames, self.frame_index / y_frames)
    }
}
