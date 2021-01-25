use rendering_ir::wgpu_helpers::{
    create_render_pipeline,
    texture::{Image, Texture},
    vertex::Vertex,
};

use super::{shader_pipeline::ShaderPipeline, *};
use crate::{
    file_system::{Asset, AssetState, FileSystem},
    gfx::{DeviceQueue, DoubleBuffer},
    EventQueue,
};
use rendering_ir::camera3d::Camera3d;

mod model;
mod uniforms;
use uniforms::UniformContainer;
mod instance;
mod light;

const MAX_INSTANCES: u32 = 1000;

pub struct ScenePipeline {
    pipeline: wgpu::RenderPipeline,
    gfx_settings: GfxSettings,
    uniforms: UniformContainer,
    camera: Camera3d,
    model: model::Model,
    model_texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl ShaderPipeline for ScenePipeline {
    fn new(
        render_texture_format: Option<wgpu::TextureFormat>,
        gfx_settings: GfxSettings,
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let camera = {
            let (w, h) = gfx_settings.render_resolution;
            let (w, h) = (w as f32, h as f32);

            let camera = Camera3d::new(w, h, [0.0; 3], [0.0; 3], false);

            camera
        };

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let uniforms = UniformContainer::new(
            &camera,
            &uniform_bind_group_layout,
            &DeviceQueue { device, queue },
        );

        let model_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // Diffuse texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Uint,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                        count: None,
                    },
                    // Normal map
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                        count: None,
                    },
                ],
                label: Some("model_texture_bind_group_layout"),
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout, &model_texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_texture_format = match render_texture_format {
            Some(r) => r,
            None => {
                panic!("Required render texture format to be Some(format)!");
            }
        };

        let pipeline = create_render_pipeline(
            &device,
            &render_pipeline_layout,
            Some("Render Pipeline"),
            render_texture_format,
            Some(Texture::DEPTH_FORMAT),
            &[model::ModelVertex::desc(), instance::Instance::desc()],
            wgpu::include_spirv!("../../../gfx/shaders/spv/obj_scene.vert.spv"),
            wgpu::include_spirv!("../../../gfx/shaders/spv/obj_scene.frag.spv"),
        );

        let res_dir = FileSystem::res_dir();
        let model = model::Model::load(
            &DeviceQueue { device, queue },
            res_dir.join("monkey_blend").join("monkey.obj"),
            &model_texture_bind_group_layout,
            MAX_INSTANCES,
            true,
        );

        Self {
            model,
            gfx_settings,
            pipeline,
            camera,
            uniforms,
            model_texture_bind_group_layout,
        }
    }

    fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    fn gfx_settings(&self) -> GfxSettings {
        self.gfx_settings
    }

    fn replace_gfx_settings(&mut self, new_settings: GfxSettings) {
        self.gfx_settings = new_settings;
    }

    fn update(
        &mut self,
        dq: &DeviceQueue,
        command_queue: &RenderQueue,
        event_queue: &mut EventQueue,
    ) {
        // Do any resets
        self.model.clear_instances();

        // Process commands
        for cmd in command_queue.commands() {
            match cmd {
                RenderCommand::CameraUpdate {
                    target,
                    eye,
                    orthographic,
                } => {
                    self.camera.update(*target, *eye, *orthographic);
                }
                RenderCommand::Asset(asset_msg) => match asset_msg {
                    AssetCommand::LoadObj {
                        file,
                        max_instances,
                    } => {
                        println!("TODO: load obj file.");
                    }
                    AssetCommand::DropObj { file } => {
                        println!("TODO: drop obj file.");
                    }
                    _ => {}
                },
                RenderCommand::ModelDraw {
                    file,
                    position,
                    rotation,
                    scale,
                } => {
                    // Instancing
                    let instance = instance::Instance::new(*position, *rotation, *scale);
                    self.model.add_instance(instance);
                }
                _ => {}
            }
        }

        // Update uniforms
        self.uniforms.update(&self.camera);

        // Update buffers
        self.model.update_buffers(dq);
        self.uniforms.update_buffer(dq);
    }

    fn render<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>) {
        // NOTE: instead of abstracting things away, treat this as a 'shader pipeline', a self contained render system with all draw calls at the top level.
        // This keeps it apparent what each shader is doing and allows you to easily compare to the GLSL shader.

        // Set the viewport to the render resolution
        let window_res = self.gfx_settings.physical_resolution;
        let render_res = self.gfx_settings.render_resolution;

        let (vdepth_min, vdepth_max) = Self::viewport_depth_values();

        render_pass.set_viewport(
            0.,
            0.,
            render_res.0 as f32,
            render_res.1 as f32,
            vdepth_min,
            vdepth_max,
        );
        // Make draw calls.
        {
            render_pass.set_bind_group(0, &self.uniforms.bind_group(), &[]);
            let model = &self.model;
            for mesh in model.meshes() {
                if mesh.instances.is_empty() {
                    continue;
                }

                render_pass.set_bind_group(1, &model.material(mesh.material).bind_group, &[]);

                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, mesh.instances.buffer().slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..));
                render_pass.draw_indexed(0..mesh.index_len, 0, 0..mesh.instances.len() as _);
            }
        }
        // Set it back to the full resolution
        render_pass.set_viewport(
            0.,
            0.,
            window_res.0 as f32,
            window_res.1 as f32,
            vdepth_min,
            vdepth_max,
        );
    }

    fn resize(&mut self, old_settings: GfxSettings, new_settings: GfxSettings, dq: &DeviceQueue) {
        self.gfx_settings = new_settings;

        let (w, h) = self.gfx_settings.render_resolution;
        self.camera.resize(w as f32, h as f32);

        let (w, h) = self.gfx_settings.physical_resolution;
        self.uniforms.update_screen_size(w as f32, h as f32);
    }
}
