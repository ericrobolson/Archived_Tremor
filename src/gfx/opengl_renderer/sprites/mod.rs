use std::collections::HashMap;
use std::ffi::CString;
extern crate gl;

use super::*;
use super::{texture::Texture, vertices::Vertices};

//use crate::ecs::World;
//use crate::lib_core::Entity;
mod sprite;
pub use sprite::Sprite;

pub struct SpritePass {
    program: program::Program,
    vao: vao::Vao,
    vbo: vbo::Vbo,
    //sprite_to_entity_map: HashMap<Entity, Sprite>,
    textures: HashMap<gl::types::GLuint, Texture>,
}

impl SpritePass {
    pub fn new() -> Self {
        let vert_shader = shaders::Shader::from_vert_source(
            &CString::new(include_str!("..\\shader_implementations\\sprite.vert")).unwrap(),
        )
        .unwrap();

        let frag_shader = shaders::Shader::from_frag_source(
            &CString::new(include_str!("..\\shader_implementations\\sprite.frag")).unwrap(),
        )
        .unwrap();

        let shader_program = program::Program::from_shaders(&[vert_shader, frag_shader]).unwrap();

        Self {
            program: shader_program,
            vao: vao::Vao::new(),
            vbo: vbo::Vbo::new(),
            // sprite_to_entity_map: HashMap::new(),
            textures: HashMap::new(),
        }
    }

    pub fn render(&mut self, resolution: Resolution) {
        /////////////
        // Cleanup //
        /////////////
        /*
        for entity_to_delete in world.get_deleted_entities() {
            //     self.sprite_to_entity_map.remove(entity_to_delete);
        }
        */

        //TODO: if textures have no references, remove it
        //for (tex_id, texture) in self.textures.iter(){
        // //TODO: delete
        //}

        /////////////////////////////////
        // Prep all data for rendering //
        /////////////////////////////////
        self.program.set_used();
        /*
        for entity in world.entities() {
            let sprite = &world.sprites[*entity];

            match sprite {
                Some(sprite) => {
                    // Ensure texture exists
                    let tex_id = {
                        let mut existing_texture = None;

                        // Check to see if the texture exists
                        for (tex_id, texture) in self.textures.iter() {
                            if texture.name() == sprite.texture.to_string() {
                                existing_texture = Some(tex_id);
                                break;
                            }
                        }

                        match existing_texture {
                            Some(t) => *t,
                            _ => {
                                // If it doesn't exist, create a new one.
                                let t = texture::Texture::new(sprite.texture.to_string());
                                let tex_id = t.id();
                                self.textures.insert(t.id(), t);

                                tex_id
                            }
                        }
                    };

                    // Init the sprite
                    // TODO: possibly look into just updating the existing sprite.
                    let tex = self.textures.get(&tex_id).unwrap();

                    let mut sprite = sprite::Sprite::new(&tex, None, None, None, None);

                    // Update the sprite's position
                    let pos2d = &world.positions[*entity];

                    match pos2d {
                        Some(position) => {
                            let (pos_x, pos_y, pos_z) = interpolate_velocity(
                                *entity,
                                position.x.into(),
                                position.y.into(),
                                position.z.into(),
                                world,
                            );

                            sprite.set_position(pos_x, pos_y);
                        }
                        _ => {}
                    }

                    //   self.sprite_to_entity_map.insert(*entity, sprite);
                }
                _ => {}
            }
        }
        */

        // TODO: optimize so that it only updates when changed? Not doing for now.

        // Batch all sprites by texture
        let mut sprite_vert_map = HashMap::<gl::types::GLuint, Vertices>::new();
        /*
        for sprite in self.sprite_to_entity_map.values() {
            let mut verts = sprite.into_verts(resolution);

            if sprite_vert_map.contains_key(&sprite.texture_id()) {
                let existing_verts = sprite_vert_map.get(&sprite.texture_id());

                match existing_verts {
                    Some(existing) => {
                        verts.join(existing);
                    }
                    _ => {}
                }
            }

            sprite_vert_map.insert(sprite.texture_id(), verts);
        }
        */
        //////////////////////////////////
        // Following is render specific //
        //////////////////////////////////
        unsafe {
            // Enables alpha
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::MULTISAMPLE);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
        }

        // Render all sprite verts by texture
        for (tex_id, verts) in sprite_vert_map.into_iter() {
            let tex = self.textures.get(&tex_id);

            match tex {
                Some(texture) => {
                    texture.bind();

                    self.vao.buffer(&mut self.vbo, &verts);
                    self.vao.render(&self.vbo);

                    texture.unbind();
                }
                _ => {
                    println!("Attempted to use an unbound texture!");
                }
            }
        }
    }
}
