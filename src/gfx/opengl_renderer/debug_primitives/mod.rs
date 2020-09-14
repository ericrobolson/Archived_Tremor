use std::collections::HashMap;
use std::ffi::CString;
extern crate gl;

use super::*;
use super::{texture::Texture, vertices::Vertices};
//use crate::ecs::World;
//use crate::lib_core::{primitives::Aabb, primitives::Circle, Entity};

mod circle;
mod rectangle;
use rectangle::Rectangle;

pub struct DebugPass {
    program: program::Program,
    vao: vao::Vao,
    vbo: vbo::Vbo,
}

impl DebugPass {
    pub fn new() -> Self {
        let vert_shader = shaders::Shader::from_vert_source(
            &CString::new(include_str!("..\\shader_implementations\\triangle.vert")).unwrap(),
        )
        .unwrap();

        let frag_shader = shaders::Shader::from_frag_source(
            &CString::new(include_str!("..\\shader_implementations\\triangle.frag")).unwrap(),
        )
        .unwrap();

        let shader_program = program::Program::from_shaders(&[vert_shader, frag_shader]).unwrap();

        Self {
            program: shader_program,
            vao: vao::Vao::new(),
            vbo: vbo::Vbo::new(),
        }
    }
    pub fn render(&mut self, resolution: Resolution) {
        /////////////////////////////////
        // Prep all data for rendering //
        /////////////////////////////////
        self.program.set_used();

        let mut rects: Vec<rectangle::Rectangle> = vec![];
        let mut circles: Vec<circle::Circle> = vec![];
        /*
        for entity in world.entities() {
            let entity = *entity;

            // Get rects
            let rect = get_rect(entity, world);
            match rect {
                Some(rect) => {
                    rects.push(rect);
                }
                None => {}
            }

            // Get circle
            let circle = get_circle(entity, world);
            match circle {
                Some(circle) => {
                    circles.push(circle);
                }
                None => {}
            }
        }
        */

        let mut verts: Option<Vertices> = None;
        for rect in rects.iter() {
            verts = match verts {
                Some(v) => {
                    let mut v2 = rect.into_verts(resolution);
                    v2.join(&v);
                    Some(v2)
                }
                _ => Some(rect.into_verts(resolution)),
            };
        }

        for circle in circles.iter() {
            verts = match verts {
                Some(v) => {
                    let mut v2 = circle.into_verts(resolution);
                    v2.join(&v);
                    Some(v2)
                }
                _ => Some(circle.into_verts(resolution)),
            };
        }

        //////////////////////////////////
        // Following is render specific //
        //////////////////////////////////
        unsafe {
            // Enables alpha
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::MULTISAMPLE);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        }

        // Render all sprite verts by texture
        match verts {
            Some(verts) => {
                self.vao.buffer(&mut self.vbo, &verts);
                self.vao.render(&self.vbo);
            }
            _ => {}
        }
    }
}
/*
fn get_circle(entity: Entity, world: &World) -> Option<circle::Circle> {
    let circle = &world.circles[entity];
    match circle {
        Some(circle) => {
            let mut c = circle::Circle::new();
            c.set_size(circle.radius().into());
            c.set_color(1.0, 0.0, 0.0);

            let pos2d = &world.positions[entity];

            match pos2d {
                Some(position) => {
                    // interpolate velocity
                    let (pos_x, pos_y, _) = interpolate_velocity(
                        entity,
                        position.x.into(),
                        position.y.into(),
                        position.z.into(),
                        world,
                    );

                    c.set_position(pos_x, pos_y);
                }
                _ => {}
            }
            return Some(c);
        }
        None => {
            return None;
        }
    }
}

fn get_rect(entity: Entity, world: &World) -> Option<Rectangle> {
    let aabb = &world.aabbs[entity];
    match aabb {
        Some(aabb) => {
            let pos2d = &world.positions[entity];
            let mut rect = Rectangle::new();

            match pos2d {
                Some(position) => {
                    // interpolate velocity
                    let (pos_x, pos_y, _) = interpolate_velocity(
                        entity,
                        position.x.into(),
                        position.y.into(),
                        position.z.into(),
                        world,
                    );

                    rect.set_position(pos_x, pos_y);
                }
                _ => {}
            }

            let size = aabb.size();

            rect.set_size((size.x).into(), (size.y).into());

            rect.set_color(1.0, 0.0, 0.0);

            match &world.static_bodies[entity] {
                Some(_) => {
                    rect.set_color(0.0, 1.0, 1.0);
                }
                _ => {}
            }

            match &world.rigid_bodies[entity] {
                Some(_) => {
                    rect.set_color(0.0, 1.0, 0.0);
                }
                _ => {}
            }

            return Some(rect);
        }
        None => {
            return None;
        }
    }
}
*/
