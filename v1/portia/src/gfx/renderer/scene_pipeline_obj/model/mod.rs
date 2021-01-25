use super::instance;
use crate::gfx::DeviceQueue;
use rayon::prelude::*;
use rendering_ir::wgpu_helpers::{
    texture::{Image, Texture},
    vertex::{Index, Vertex},
};
use std::path::Path;
use wgpu::util::DeviceExt;

// inspired by https://sotrh.github.io/learn-wgpu/beginner/tutorial9-models/#loading-models-with-tobj

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
    normal: [f32; 3],
    tangent: [f32; 3],
    bitangent: [f32; 3],
}

impl Vertex for ModelVertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                // tex coords
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
                // normal
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float3,
                },
                // tangent
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float3,
                },
                // bitangent
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }
}

/// A self contained 3d object (or objects) to render.
pub struct Model {
    meshes: Vec<Mesh>,
    materials: Vec<Material>,
}

pub struct Material {
    name: String,
    pub diffuse_texture: Texture,
    pub normal_texture: Texture,
    pub bind_group: wgpu::BindGroup,
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_len: u32,
    pub vert_len: u32,
    pub material: usize,
    pub instances: instance::InstanceContainer,
}

impl Model {
    pub fn meshes(&self) -> &Vec<Mesh> {
        &self.meshes
    }

    pub fn material(&self, index: usize) -> &Material {
        &self.materials[index]
    }

    pub fn clear_instances(&mut self) {
        for mesh in self.meshes.iter_mut() {
            mesh.instances.clear();
        }
    }

    pub fn add_instance(&mut self, instance: instance::Instance) {
        for mesh in self.meshes.iter_mut() {
            mesh.instances.add_instance(instance);
        }
    }

    pub fn update_buffers(&mut self, dq: &DeviceQueue) {
        for mesh in self.meshes.iter_mut() {
            mesh.instances.update_buffer(dq);
        }
    }

    pub fn load<P: AsRef<Path>>(
        dq: &DeviceQueue,
        path: P,
        model_tex_bind_group_layout: &wgpu::BindGroupLayout,
        max_instances: u32,
        filtered_textures: bool,
    ) -> Self {
        let (obj_models, obj_materials) = tobj::load_obj(path.as_ref(), true).unwrap();
        let containing_folder = path.as_ref().parent().unwrap();

        let materials = obj_materials
            .par_iter()
            .map(|mat| {
                println!("A MAT! {:?}", mat.diffuse_texture);
                let texture = {
                    let diffuse_path = &mat.diffuse_texture;
                    let path = containing_folder.join(diffuse_path);
                    Texture::load(dq.device, dq.queue, path, filtered_textures, false).unwrap()
                };

                let normal_texture = {
                    let normal_path = &mat.normal_texture;
                    let path = containing_folder.join(normal_path);
                    Texture::load(dq.device, dq.queue, path, filtered_textures, true).unwrap()
                };

                let bind_group = {
                    dq.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &model_tex_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(&texture.view()),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(&texture.sampler()),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: wgpu::BindingResource::TextureView(
                                    &normal_texture.view(),
                                ),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: wgpu::BindingResource::Sampler(&normal_texture.sampler()),
                            },
                        ],
                        label: Some(&mat.name),
                    })
                };

                Material {
                    name: mat.name.clone(),
                    diffuse_texture: texture,
                    normal_texture,
                    bind_group,
                }
            })
            .collect::<Vec<Material>>();

        let meshes = obj_models
            .par_iter()
            .map(|m| {
                // Verts + indexes
                let mut verts = vec![];
                for i in 0..m.mesh.positions.len() / 3 {
                    verts.push(ModelVertex {
                        position: [
                            m.mesh.positions[i * 3],
                            m.mesh.positions[i * 3 + 1],
                            m.mesh.positions[i * 3 + 2],
                        ],
                        tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
                        normal: [
                            m.mesh.normals[i * 3],
                            m.mesh.normals[i * 3 + 1],
                            m.mesh.normals[i * 3 + 2],
                        ],
                        // Empty values for tangent + bitangent, will calculate later
                        tangent: [0.0; 3].into(),
                        bitangent: [0.0; 3].into(),
                    });
                }

                // Calculate tangents + bitangents
                for c in m.mesh.indices.chunks(3) {
                    let v0 = verts[c[0] as usize];
                    let v1 = verts[c[1] as usize];
                    let v2 = verts[c[2] as usize];

                    let pos0: cgmath::Vector3<_> = v0.position.into();
                    let pos1: cgmath::Vector3<_> = v1.position.into();
                    let pos2: cgmath::Vector3<_> = v2.position.into();

                    let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
                    let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
                    let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

                    // Calculate the edges of the triangle
                    let delta_pos1 = pos1 - pos0;
                    let delta_pos2 = pos2 - pos0;

                    // This will give us a direction to calculate the
                    // tangent and bitangent
                    let delta_uv1 = uv1 - uv0;
                    let delta_uv2 = uv2 - uv0;

                    // Solving the following system of equations will
                    // give us the tangent and bitangent.
                    //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
                    //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
                    // Luckily, the place I found this equation provided
                    // the solution!
                    let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                    let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                    let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r;
                    // We'll use the same tangent/bitangent for each vertex in the triangle
                    verts[c[0] as usize].tangent = tangent.into();
                    verts[c[1] as usize].tangent = tangent.into();
                    verts[c[2] as usize].tangent = tangent.into();

                    verts[c[0] as usize].bitangent = bitangent.into();
                    verts[c[1] as usize].bitangent = bitangent.into();
                    verts[c[2] as usize].bitangent = bitangent.into();
                }

                let vert_len = verts.len() as u32;
                let vertex_buffer =
                    dq.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(&format!("{:?} Vertex Buffer", m.name)),
                            contents: bytemuck::cast_slice(&verts),
                            usage: wgpu::BufferUsage::VERTEX,
                        });

                let index_buffer = {
                    let indexes: Vec<Index> = m.mesh.indices.iter().map(|i| *i as Index).collect();
                    dq.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(&format!("{:?} Index Buffer", m.name)),
                            contents: bytemuck::cast_slice(&indexes),
                            usage: wgpu::BufferUsage::INDEX,
                        })
                };

                // Instances
                let instances = {
                    let container = instance::InstanceContainer::new(max_instances, dq);
                    container
                };

                Mesh {
                    name: m.name.clone(),
                    vertex_buffer,
                    index_buffer,
                    index_len: m.mesh.indices.len() as u32,
                    vert_len,
                    material: m.mesh.material_id.unwrap_or(0),
                    instances,
                }
            })
            .collect::<Vec<Mesh>>();

        Self { meshes, materials }
    }
}
