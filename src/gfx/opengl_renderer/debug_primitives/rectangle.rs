use super::*;
use super::{texture::Texture, vertices::Vertices};

extern crate gl;

#[derive(Clone)]
pub struct Rectangle {
    width: f32,
    height: f32,
    position_x: f32,
    position_y: f32,
    r: f32,
    g: f32,
    b: f32,

    visible: bool,
}

impl Rectangle {
    pub fn new() -> Self {
        Self {
            width: 10.0,
            height: 10.0,
            position_x: 0.0,
            position_y: 0.0,
            r: 1.0,
            g: 1.0,
            b: 1.0,
            visible: true,
        }
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position_x = x;
        self.position_y = y;
    }

    pub fn set_color(&mut self, r: f32, g: f32, b: f32) {
        self.r = r;
        self.g = g;
        self.b = b;
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
        let half_width = (self.width) / 2.0;
        let half_height = (self.height) / 2.0;

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

        // Generate the verts
        let quad_verts: Vec<f32> = vec![
            // Positions // Colors
            // Triangle 1
            start_x, start_y, z, r, g, b, // bottom left
            start_x, end_y, z, r, g, b, // top left
            end_x, end_y, z, r, g, b, // top right
            // Triangle 2
            end_x, end_y, z, r, g, b, // top right
            end_x, start_y, z, r, g, b, // bottom right
            start_x, start_y, z, r, g, b, // bottom left
        ];

        let verts = quad_verts;

        let vertices = Vertices::new(verts, vec![VERT_POSITION_LEN, COLOR_POSITION_LEN]);

        return vertices;
    }
}
