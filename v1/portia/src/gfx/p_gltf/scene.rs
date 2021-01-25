use game_math::f32::*;
use std::rc::Rc;
use wgpu::util::DeviceExt;

use super::{
    animations::Animation,
    boundingbox::BoundingBox,
    manager::LoadableGltf,
    material::Material,
    mesh::Mesh,
    node::Node,
    primitive::Primitive,
    skin::Skin,
    textures::{Texture, TextureSampler},
    vert_indices::{Indices, Vertex},
};
use crate::gfx::{
    uniforms::{ModelUbo, ModelUniformContainer, NodeUbo, NodeUniformContainer, MAX_NUM_JOINTS},
    DeviceQueue,
};
use data_structures::{Hierarchy, Id};

pub struct Scene {
    nodes: Hierarchy<Node>,
    vertices: Vec<Vertex>,
    model_ubo: ModelUniformContainer,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    /*
    indices: Indices,
    aabb: Mat4,
    textures: Vec<Texture>,
    texture_samplers: Vec<TextureSampler>,
    materials: Vec<Material>,
    animations: Vec<Animation>,
    extensions: Vec<String>,
    */
}
impl Scene {
    /// Buffer all data.
    pub fn buffer(&mut self) {
        for node in self.nodes.iter_items_mut() {
            node.buffer();
        }
    }

    pub fn initialize(
        data: LoadableGltf,
        model_layout: &wgpu::BindGroupLayout,
        node_layout: &wgpu::BindGroupLayout,
        material_layout: &wgpu::BindGroupLayout,
        dq: &DeviceQueue,
    ) -> Self {
        let scale = 1.0;

        // https://github.com/SaschaWillems/Vulkan-glTF-PBR/blob/master/base/VulkanglTFModel.hpp#L1149
        load_texture_samplers();
        load_textures();
        load_materials();

        let mut nodes = Hierarchy::<Node>::new();
        let mut index_buffer = vec![];
        let mut vertex_buffer = vec![];

        let scene = {
            match data.document.default_scene() {
                Some(s) => s,
                None => {
                    if data.document.scenes().len() > 1 {
                        // TODO: how to handle multiple scenes?
                        panic!("Multiple scenes are not supported at this moment.");
                    }
                    data.document.scenes().nth(0).unwrap()
                }
            }
        };

        // load the nodes in the scene
        // TODO: should this filter out nodes with children? Need to confirm that it's only top level nodes.
        for node in scene.nodes() {
            load_node(
                &node,
                &data,
                &mut nodes,
                None,
                &mut index_buffer,
                &mut vertex_buffer,
                scale,
                node_layout,
                material_layout,
                dq,
            );
        }

        // load animations
        if data.document.animations().len() > 0 {
            load_animations();
        }
        // Load + assign skins
        for node in nodes.iter_items_mut() {
            node.update();
        }

        let model_ubo = ModelUbo {
            model: Mat4::identity(),
        };

        let vert_buffer = dq
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&vertex_buffer),
                usage: wgpu::BufferUsage::VERTEX,
            });

        let index_buffer = {
            dq.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&index_buffer),
                    usage: wgpu::BufferUsage::INDEX,
                })
        };

        Self {
            vertex_buffer: vert_buffer,
            index_buffer,
            nodes,
            model_ubo: ModelUniformContainer::new(model_ubo, model_layout, dq),
            vertices: vertex_buffer,
        }
    }

    /// Updates all nodes.
    pub fn update(&mut self) {
        // Process all nodes, starting with those that don't have children.
        let parents = self
            .nodes
            .iter()
            .filter(|(_id, n)| n.parent().is_none())
            .map(|(id, _n)| id)
            .collect::<Vec<Id>>();

        for parent in parents {
            self.update_node(parent);
        }
    }

    pub fn render<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>) {
        render_pass.set_bind_group(1, self.model_ubo.bind_group(), &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..));
        for node in self.nodes.iter_items() {
            match node.mesh() {
                Some(mesh) => {
                    render_pass.set_bind_group(2, node.node_ubo.bind_group(), &[]);

                    for primitive in &mesh.primitives {
                        render_pass.set_bind_group(
                            3,
                            primitive.material.material_ubo.bind_group(),
                            &[],
                        );

                        let index_start = primitive.first_index;
                        let index_end = index_start + primitive.index_count;

                        render_pass.draw_indexed(
                            index_start..index_end,
                            mesh.base_vertex as i32,
                            0..1 as _,
                        );
                    }
                }
                None => {}
            }
        }
    }

    /// Updates an individual node and its children.
    fn update_node(&mut self, id: Id) {
        if let Some(ref mut node) = self.nodes.node_mut(id) {
            node.item_mut().update();
            for child in node.children() {
                self.update_node(child);
            }
        }
    }
}

type Index = u32;

fn load_node(
    gnode: &gltf::Node,
    data: &LoadableGltf,
    hierarchy: &mut Hierarchy<Node>,
    parent: Option<Id>,
    index_buffer: &mut Vec<Index>,
    vert_buffer: &mut Vec<Vertex>,
    global_scale: f32,
    node_layout: &wgpu::BindGroupLayout,
    material_layout: &wgpu::BindGroupLayout,
    dq: &DeviceQueue,
) {
    //https://github.com/SaschaWillems/Vulkan-glTF-PBR/blob/master/base/VulkanglTFModel.hpp#L661
    let name = match gnode.name() {
        Some(name) => format!("{:?}", name),
        None => format!("NODE_{:?}", hierarchy.len()),
    };

    let skin = load_skin(gnode.skin());

    // Parse transforms
    let (translation, scale, rotation) = {
        let (translation, rotation, scale) = gnode.transform().decomposed();

        let translation = Vec3::new(translation[0], translation[1], translation[2]);
        let scale = Vec3::new(scale[0], scale[1], scale[2]) * global_scale;
        let rotation = {
            let x = rotation[0];
            let y = rotation[1];
            let z = rotation[2];
            let w = rotation[3];
            Quaternion::new(x, y, z, w)
        };

        (translation, scale, rotation)
    };

    // Parse mesh
    let node_matrix = Mat4::i32(1);
    let mesh = {
        match gnode.mesh() {
            Some(mesh) => Some(process_mesh(
                &mesh,
                data,
                node_matrix,
                index_buffer,
                vert_buffer,
                global_scale,
                material_layout,
                dq,
            )),
            None => None,
        }
    };

    // Create node
    let node_uniform = NodeUbo {
        matrix: node_matrix,
        joint_matrix: [Mat4::identity(); MAX_NUM_JOINTS],
        joint_count: 0.,
    };
    let node = Node {
        node_ubo: NodeUniformContainer::new(node_uniform, node_layout, dq),
        parent,
        index: gnode.index(),
        children: vec![],
        matrix: node_matrix,
        name,
        mesh,
        skin,
        translation,
        scale,
        rotation,
    };

    let id = hierarchy.add_node(node);

    // Add parent
    match parent {
        Some(parent) => {
            hierarchy.add_parent(id, parent);
        }
        None => {}
    }

    // Process children
    for child in gnode.children() {
        load_node(
            &child,
            data,
            hierarchy,
            Some(id),
            index_buffer,
            vert_buffer,
            global_scale,
            node_layout,
            material_layout,
            dq,
        );
    }
}

fn process_mesh(
    mesh: &gltf::mesh::Mesh,
    data: &LoadableGltf,
    node_matrix: Mat4,
    index_buffer: &mut Vec<Index>,
    vert_buffer: &mut Vec<Vertex>,
    global_scale: f32,
    material_layout: &wgpu::BindGroupLayout,
    dq: &DeviceQueue,
) -> Mesh {
    // https://github.com/SaschaWillems/Vulkan-glTF-PBR/blob/master/base/VulkanglTFModel.hpp#L698

    let mut primitives: Vec<Primitive> = vec![];

    let index_start = {
        if index_buffer.len() == 0 {
            0
        } else {
            (index_buffer.len() - 1) as u32
        }
    };
    let vertex_start = {
        if vert_buffer.len() == 0 {
            0
        } else {
            (vert_buffer.len() - 1) as u32
        }
    };

    let mut index_count = 0;

    for primitive in mesh.primitives() {
        let pos_accessor = match primitive.get(&gltf::mesh::Semantic::Positions) {
            Some(positions) => positions,
            None => panic!("Positions attribute is required to load verts!"),
        };

        let pos_min = parse_accessor_vec3_value("min", &pos_accessor.min());
        let pos_max = parse_accessor_vec3_value("max", &pos_accessor.max());

        let primitive_first_index: u32 = index_count as u32;
        let mut primitive_index_count: u32 = 0;
        let primitive_vertex_count: u32 = pos_accessor.count() as u32;

        // Vertices
        {
            // START: https://github.com/SaschaWillems/Vulkan-glTF-PBR/blob/master/base/VulkanglTFModel.hpp#L712

            // 			// Skinning
            // 			// Joints
            // 			if (primitive.attributes.find("JOINTS_0") != primitive.attributes.end()) {
            // 				const tinygltf::Accessor &jointAccessor = model.accessors[primitive.attributes.find("JOINTS_0")->second];
            // 				const tinygltf::BufferView &jointView = model.bufferViews[jointAccessor.bufferView];
            // 				bufferJoints = reinterpret_cast<const uint16_t *>(&(model.buffers[jointView.buffer].data[jointAccessor.byteOffset + jointView.byteOffset]));
            // 				jointByteStride = jointAccessor.ByteStride(jointView) ? (jointAccessor.ByteStride(jointView) / sizeof(bufferJoints[0])) : tinygltf::GetTypeSizeInBytes(TINYGLTF_TYPE_VEC4);
            // 			}

            // 			if (primitive.attributes.find("WEIGHTS_0") != primitive.attributes.end()) {
            // 				const tinygltf::Accessor &weightAccessor = model.accessors[primitive.attributes.find("WEIGHTS_0")->second];
            // 				const tinygltf::BufferView &weightView = model.bufferViews[weightAccessor.bufferView];
            // 				bufferWeights = reinterpret_cast<const float *>(&(model.buffers[weightView.buffer].data[weightAccessor.byteOffset + weightView.byteOffset]));
            // 				weightByteStride = weightAccessor.ByteStride(weightView) ? (weightAccessor.ByteStride(weightView) / sizeof(float)) : tinygltf::GetTypeSizeInBytes(TINYGLTF_TYPE_VEC4);
            // 			}

            // 			hasSkin = (bufferJoints && bufferWeights);

            for v in 0..pos_accessor.count() {
                let mut vert = Vertex {
                    pos: Vec3::default().to_raw(),
                    normal: Vec3::default().to_raw(),
                    uv0: Vec2::default().to_raw(),
                    uv1: Vec2::default().to_raw(),
                    joint0: Vec4::default().to_raw(),  // TODO
                    weight0: Vec4::default().to_raw(), // TODO
                };

                vert.pos = (parse_vec3(v, &pos_accessor, data) * global_scale).to_raw();

                vert.normal = {
                    if let Some(normal_accessor) = primitive.get(&gltf::mesh::Semantic::Normals) {
                        parse_vec3(v, &normal_accessor, data).to_raw()
                    } else {
                        Vec3::default().to_raw()
                    }
                };

                vert.uv0 = {
                    if let Some(accessor) = primitive.get(&gltf::mesh::Semantic::TexCoords(0)) {
                        parse_vec2(v, &accessor, data).to_raw()
                    } else {
                        Vec2::default().to_raw()
                    }
                };

                vert.uv1 = {
                    if let Some(accessor) = primitive.get(&gltf::mesh::Semantic::TexCoords(1)) {
                        parse_vec2(v, &accessor, data).to_raw()
                    } else {
                        Vec2::default().to_raw()
                    }
                };

                // println!("TODO: read in rest of stuff.");

                // vert.joint0 = hasSkin ? glm::vec4(glm::make_vec4(&bufferJoints[v * jointByteStride])) : glm::vec4(0.0f);
                // vert.weight0 = hasSkin ? glm::make_vec4(&bufferWeights[v * weightByteStride]) : glm::vec4(0.0f);
                // // Fix for all zero weights
                // if (glm::length(vert.weight0) == 0.0f) {
                //     vert.weight0 = glm::vec4(1.0f, 0.0f, 0.0f, 0.0f);
                // }
                // vertexBuffer.push_back(vert);

                vert_buffer.push(vert);
            }

            // END: https://github.com/SaschaWillems/Vulkan-glTF-PBR/blob/master/base/VulkanglTFModel.hpp#L790
        }
        // Indices
        match primitive.indices() {
            Some(indices_accessor) => {
                // START: https://github.com/SaschaWillems/Vulkan-glTF-PBR/blob/master/base/VulkanglTFModel.hpp#L794
                let index_view = match indices_accessor.view() {
                    Some(view) => view,
                    None => unimplemented!("Expected indice accessor!"),
                };

                let buffer = {
                    let buff_index = index_view.buffer().index();
                    &data.buffers[buff_index]
                };

                let index_size = indices_accessor.size();

                let buffer_data: Vec<u8> = buffer
                    .iter()
                    .skip(index_view.offset())
                    .take(indices_accessor.count() * index_size)
                    .map(|u| *u)
                    .collect();

                primitive_index_count += indices_accessor.count() as u32;

                for i in 0..indices_accessor.count() {
                    let data: Vec<u8> = buffer_data
                        .iter()
                        .skip(i * index_size)
                        .take(index_size)
                        .map(|u| *u)
                        .collect();

                    let index = match indices_accessor.data_type() {
                        gltf::accessor::DataType::I8 => i8::from_le_bytes([data[0]]) as Index,
                        gltf::accessor::DataType::U8 => u8::from_le_bytes([data[0]]) as Index,
                        gltf::accessor::DataType::I16 => {
                            i16::from_le_bytes([data[0], data[1]]) as Index
                        }
                        gltf::accessor::DataType::U16 => {
                            u16::from_le_bytes([data[0], data[1]]) as Index
                        }
                        gltf::accessor::DataType::U32 => {
                            u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as Index
                        }
                        gltf::accessor::DataType::F32 => {
                            f32::from_le_bytes([data[0], data[1], data[2], data[3]]) as Index
                        }
                    };

                    index_count += 1;
                    index_buffer.push(index);
                }

                // END: https://github.com/SaschaWillems/Vulkan-glTF-PBR/blob/master/base/VulkanglTFModel.hpp#L832
            }
            None => {
                unimplemented!("TODO: how to handle no indices?")
            }
        }

        let material = load_material(primitive.material(), material_layout, dq);

        let mut primitive = Primitive {
            bounding_box: BoundingBox::default(),
            material,
            first_index: primitive_first_index,
            index_count: primitive_index_count,
            vertex_count: primitive_vertex_count,
        };
        primitive.set_bounding_box(pos_min, pos_max);

        primitives.push(primitive);
    }

    Mesh::new(
        primitives,
        node_matrix,
        index_start,
        index_start + index_count,
        vertex_start,
    )
}

fn load_material<'a>(
    m: gltf::Material<'a>,
    material_layout: &wgpu::BindGroupLayout,
    dq: &DeviceQueue,
) -> Material {
    use crate::gfx::uniforms::{MaterialUbo, MaterialUniformContainer};

    let pbr = m.pbr_metallic_roughness();

    let base_color_texture = {
        match pbr.base_color_texture(){
            Some(texture) => {
                println!("pbr: {:?}", texture);
                None
            },
            None => None
        }
    }; // TODO: calculate


    
    let metallic_roughness_texture = None; // TODO: calculate
    let normal_texture = None; // TODO: calculate
    let occlusion_texture = None; // TODO: calculate
    let emissive_texture = None; // TODO: calculate

    let metallic_roughness_texture_factor = if metallic_roughness_texture.is_some() {
        1.0
    } else {
        0.0
    };

    let base_color_texture_factor = if base_color_texture.is_some() {
        1.0
    } else {
        0.0
    };

    let normal_texture_factor = if normal_texture.is_some() { 1.0 } else { 0.0 };
    let occlusion_texture_factor = if occlusion_texture.is_some() {
        1.0
    } else {
        0.0
    };

    let emissive_texture_factor = if emissive_texture.is_some() { 1.0 } else { 0.0 };

    let material_ubo = MaterialUbo {
        base_color_factor: pbr.base_color_factor(),
        metallic_factor: pbr.metallic_factor(),
        roughness_factor: pbr.roughness_factor(),
        emissive_factor: m.emissive_factor(),

        metallic_roughness_texture_factor,
        base_color_texture_factor,
        normal_texture_factor,
        occlusion_texture_factor,
        emissive_texture_factor,
    };

    let material = Material {
        metallic_roughness_texture,
        base_color_texture,
        normal_texture,
        occlusion_texture,
        emissive_texture,
        material_ubo: MaterialUniformContainer::new(material_ubo, material_layout, dq),
    };

    material
}

/// Given a index, load the vertex data for a given accessor + view.
fn map_to_scalars<'a>(
    index: usize,
    accessor: &gltf::Accessor<'a>,
    view: &gltf::buffer::View<'a>,
    data: &LoadableGltf,
) -> Vec<f32> {
    let buffer = {
        let buff_index = view.buffer().index();
        &data.buffers[buff_index]
    };

    let byte_count = { view.length() / accessor.count() };

    let num_data_elements = match accessor.dimensions() {
        gltf::accessor::Dimensions::Scalar => 1,
        gltf::accessor::Dimensions::Vec2 => 2,
        gltf::accessor::Dimensions::Vec3 => 3,
        gltf::accessor::Dimensions::Vec4 => 4,
        gltf::accessor::Dimensions::Mat2 => 4,
        gltf::accessor::Dimensions::Mat3 => 9,
        gltf::accessor::Dimensions::Mat4 => 16,
    };

    let data: Vec<u8> = buffer
        .iter()
        .skip(view.offset() + byte_count * index)
        .take(byte_count)
        .map(|u| *u)
        .collect();

    let mut mapped_data = vec![];

    // Parse each data type to fit the expected one.
    let mut i = 0;
    while i < data.len() {
        let value = match accessor.data_type() {
            gltf::accessor::DataType::I8 => {
                let v = i8::from_le_bytes([data[i]]) as f32;
                i += 1;
                v
            }
            gltf::accessor::DataType::U8 => {
                let v = u8::from_le_bytes([data[i]]) as f32;
                i += 1;
                v
            }
            gltf::accessor::DataType::I16 => {
                let v = i16::from_le_bytes([data[i], data[i + 1]]) as f32;
                i += 2;
                v
            }
            gltf::accessor::DataType::U16 => {
                let v = u16::from_le_bytes([data[i], data[i + 1]]) as f32;
                i += 2;
                v
            }
            gltf::accessor::DataType::U32 => {
                let v = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as f32;
                i += 4;
                v
            }
            gltf::accessor::DataType::F32 => {
                // Do a boundary check
                let n0 = data[i];
                let n1 = data[i + 1];
                let n2 = data[i + 2];
                let n3 = data[i + 3];
                let v = f32::from_le_bytes([n0, n1, n2, n3]) as f32;
                i += 4;
                v
            }
        };

        mapped_data.push(value);
    }

    mapped_data
}

fn parse_vec2<'a>(vertex_index: usize, accessor: &gltf::Accessor<'a>, data: &LoadableGltf) -> Vec2 {
    let mut value = Vec2::default();
    let view = match accessor.view() {
        Some(view) => view,
        None => {
            println!("NO TEXTURE!");
            return value;
        }
    };

    let values = map_to_scalars(vertex_index, accessor, &view, data);
    value.x = values[0];
    value.y = values[1];

    value
}

fn parse_vec3<'a>(vertex_index: usize, accessor: &gltf::Accessor<'a>, data: &LoadableGltf) -> Vec3 {
    let mut value = Vec3::default();

    let normal_view = match accessor.view() {
        Some(view) => view,
        None => {
            panic!("Normal view is required! TODO: how to handle no value?")
        }
    };

    let values = map_to_scalars(vertex_index, accessor, &normal_view, data);
    value.x = values[0];
    value.y = values[1];
    value.z = values[2];

    value
}

fn parse_accessor_vec3_value(name: &'static str, value: &Option<gltf::json::Value>) -> Vec3 {
    match value {
        Some(value) => match value {
            gltf::json::Value::Array(array) => {
                let parse_number = |v| match v {
                    gltf::json::Value::Number(n) => n.as_f64().unwrap() as f32,
                    _ => {
                        println!("{:#?}", v);
                        unimplemented!("Unhandled number mapping! {:#?}", v)
                    }
                };

                let x = parse_number(array[0].clone());
                let y = parse_number(array[1].clone());
                let z = parse_number(array[2].clone());
                Vec3::new(x, y, z)
            }
            _ => {
                println!(
                    "Provided {:?} for '{:?}'. Setting to Vec3::default().",
                    value, name
                );
                Vec3::default()
            }
        },
        None => {
            println!(
                "Didn't recieve a valid value for {:?}. Was provided {:?}.",
                name, value
            );

            Vec3::default()
        }
    }
}

fn load_skin(skin: Option<gltf::Skin>) -> Option<Skin> {
    match skin {
        Some(skin) => {
            unimplemented!("Need to handle skins")
        }
        None => None,
    }
}

fn load_texture_samplers() {
    println!("TODO: texture samplers");
}
fn load_textures() {
    println!("TODO: textures");
}
fn load_materials() {
    println!("TODO: textures");
}
fn load_animations() {
    println!("TODO: animations");
}
