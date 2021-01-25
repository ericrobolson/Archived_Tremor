use image::{DynamicImage, GenericImageView};
use std::path::Path;

// inspired by https://sotrh.github.io/learn-wgpu/beginner/tutorial9-models/#loading-models-with-tobj

/// The type for Indexes in the model.
pub type Index = u32;

/// Data a Vertex contains.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

#[derive(Clone)]
/// A self contained 3d object (or objects) to render.
pub struct Model {
    /// All meshes that make up the model
    pub meshes: Vec<Mesh>,
    /// All materials that make up the model
    pub materials: Vec<Material>,
    /// The minimum AABB point
    pub aabb_min: [f32; 3],
    /// The maximum AABB point
    pub aabb_max: [f32; 3],
}

#[derive(Clone)]
/// A mesh containing a material + verts and indices.
pub struct Mesh {
    /// The name of the mesh
    pub name: String,
    /// The Vertexes of the mesh
    pub vertexes: Vec<Vertex>,
    /// The Indices of the mesh
    pub indices: Vec<u32>,
    /// The index of the material on the Model.
    pub material_index: Option<usize>,
}

#[derive(Clone)]
pub struct Material {
    pub name: String,
    pub diffuse_texture: Option<Texture>,
    pub normal_texture: Option<Texture>,
    pub ambient_texture: Option<Texture>,
    pub specular_texture: Option<Texture>,
    pub shininess_texture: Option<Texture>,
    pub dissolve_texture: Option<Texture>,

    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub shininess: f32,
    /// Dissolve attribute is the alpha term for the material.
    pub dissolve: f32,
    /// Optical density aka index of refraction. Called optical_density in the MTL spec.
    pub optical_density: f32,
}

/// Options that may be performed when loading a model
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LoadingOptions {
    /// Adjust the model so it is centered at [0,0,0]
    CenterModel,
    /// Scale the model so it is in the bounds of [-1,-1,-1] to [1,1,1]
    Normalize,
}

#[derive(Clone)]
pub struct Texture {
    pub image: DynamicImage,
}

impl Model {
    pub fn meshes(&self) -> &Vec<Mesh> {
        &self.meshes
    }

    pub fn material(&self, index: Option<usize>) -> Option<&Material> {
        match index {
            Some(index) => {
                if index < self.meshes.len() {
                    Some(&self.materials[index])
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Loads a Wavefront OBJ file from the given path
    pub fn load<P: AsRef<Path>>(path: P, loading_options: Vec<LoadingOptions>) -> Self {
        let (obj_models, obj_materials) = tobj::load_obj(path.as_ref(), true).unwrap();
        let containing_folder = path.as_ref().parent().unwrap();

        let mut min = [0.; 3];
        let mut max = [0.; 3];

        let materials = obj_materials
            .iter()
            .map(|mat| {
                let diffuse_texture = parse_texture(&mat.diffuse_texture, containing_folder);
                let normal_texture = parse_texture(&mat.normal_texture, containing_folder);
                let ambient_texture = parse_texture(&mat.ambient_texture, containing_folder);
                let specular_texture = parse_texture(&mat.specular_texture, containing_folder);
                let shininess_texture = parse_texture(&mat.shininess_texture, containing_folder);
                let dissolve_texture = parse_texture(&mat.dissolve_texture, containing_folder);

                let ambient = mat.ambient;
                let diffuse = mat.diffuse;
                let specular = mat.specular;
                let shininess = mat.shininess;
                let dissolve = mat.dissolve;
                let optical_density = mat.optical_density;

                Material {
                    name: mat.name.clone(),
                    diffuse_texture,
                    normal_texture,
                    ambient_texture,
                    specular_texture,
                    shininess_texture,
                    dissolve_texture,
                    ambient,
                    diffuse,
                    specular,
                    shininess,
                    dissolve,
                    optical_density,
                }
            })
            .collect::<Vec<Material>>();

        let mut meshes = obj_models
            .iter()
            .map(|m| {
                // Verts + indexes
                let mut verts = vec![];
                for i in 0..m.mesh.positions.len() / 3 {
                    // Ensure texcoords aren't empty
                    let tex_coords = {
                        if m.mesh.texcoords.len() > 0 {
                            [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]]
                        } else {
                            [0.; 2]
                        }
                    };

                    let position = {
                        [
                            m.mesh.positions[i * 3],
                            m.mesh.positions[i * 3 + 1],
                            m.mesh.positions[i * 3 + 2],
                        ]
                    };

                    // Update min/max values
                    for index in 0..3 {
                        if position[index] < min[index] {
                            min[index] = position[index];
                        }

                        if position[index] > max[index] {
                            max[index] = position[index];
                        }
                    }

                    let normal = {
                        if m.mesh.normals.len() > 0 {
                            [
                                m.mesh.normals[i * 3],
                                m.mesh.normals[i * 3 + 1],
                                m.mesh.normals[i * 3 + 2],
                            ]
                        } else {
                            unimplemented!("TODO: need to handle case with no normal.s");
                        }
                    };

                    verts.push(Vertex {
                        position,
                        tex_coords,
                        normal,
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

                let indices: Vec<Index> = m.mesh.indices.iter().map(|i| *i as Index).collect();
                let vertexes = verts;
                Mesh {
                    name: m.name.clone(),
                    vertexes,
                    indices,
                    material_index: m.mesh.material_id,
                }
            })
            .collect::<Vec<Mesh>>();

        for option in &loading_options {
            unimplemented!("TODO: Loading option {:?}", option);

            match option {
                LoadingOptions::CenterModel => {
                    let avg = |a, b| (b - a) / 2.;

                    let xdelta = avg(min[0], max[0]);
                    let ydelta = avg(min[1], max[1]);
                    let zdelta = avg(min[2], max[2]);

                    let mut min = [0.0; 3];
                    let mut max = min;

                    for mesh in meshes.iter_mut() {
                        for vert in mesh.vertexes.iter_mut() {
                            vert.position[0] -= xdelta;
                            vert.position[1] -= ydelta;
                            vert.position[2] -= zdelta;

                            // Update min/max values
                            for index in 0..3 {
                                let position = vert.position;

                                if position[index] < min[index] {
                                    min[index] = position[index];
                                }

                                if position[index] > max[index] {
                                    max[index] = position[index];
                                }
                            }
                        }
                    }
                }
                LoadingOptions::Normalize => {
                    unimplemented!("TODO: Loading option {:?}", option);
                }
            }
        }

        Self {
            meshes,
            materials,
            aabb_min: min,
            aabb_max: max,
        }
    }
}

fn parse_texture(file: &String, containing_folder: &Path) -> Option<Texture> {
    let file = file.trim();

    if file.len() == 0 {
        return None;
    }
    let path = containing_folder.join(file);

    let image = match image::open(path) {
        Ok(i) => i,
        Err(e) => {
            println!("ImageError: {:?}. Returning None.", e);
            return None;
        }
    };

    Some(Texture { image })
}
