use super::*;
use super::{texture::Texture, vertices::Vertices};

use std::f32::consts::PI;

extern crate gl;

#[derive(Clone)]
pub struct Circle {
    radius: f32,
    position_x: f32,
    position_y: f32,
    segments: usize,
    r: f32,
    g: f32,
    b: f32,

    visible: bool,
}

impl Circle {
    pub fn new() -> Self {
        Self {
            radius: 100.0,
            segments: 8,
            position_x: 0.0,
            position_y: 0.0,
            r: 1.0,
            g: 1.0,
            b: 1.0,
            visible: true,
        }
    }

    pub fn set_size(&mut self, radius: f32) {
        self.radius = radius;
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
        // Placeholder for zindex
        let z = 0.0;

        // Initialize the colors
        let r = self.r;
        let g = self.g;
        let b = self.b;

        // Initialize the vertices
        const VERT_POSITION_LEN: usize = 3;
        const COLOR_POSITION_LEN: usize = 3;

        let mut verts = vec![];

        for i in 0..self.segments {
            // Center vert
            let mut v = vec![self.position_x / res_w, self.position_y / res_h, z, r, g, b];
            verts.append(&mut v);

            // First vert
            let theta = 2.0 * PI * (i as f32) / (self.segments as f32);
            let x = (self.position_x + self.radius * f32::cos(theta)) / res_w;
            let y = (self.position_y + self.radius * f32::sin(theta)) / res_h;

            let mut v = vec![x, y, z, r, g, b];
            verts.append(&mut v);

            // Second vert
            let wrapped_i = i + 1;
            let theta = 2.0 * PI * (wrapped_i as f32) / (self.segments as f32);
            let x = (self.position_x + self.radius * f32::cos(theta)) / res_w;
            let y = (self.position_y + self.radius * f32::sin(theta)) / res_h;

            let mut v = vec![x, y, z, r, g, b];
            verts.append(&mut v);
        }

        let vertices = Vertices::new(verts, vec![VERT_POSITION_LEN, COLOR_POSITION_LEN]);

        return vertices;
    }
}
