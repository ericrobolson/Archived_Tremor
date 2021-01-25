use super::{shader_pipeline::ShaderPipeline, *};
use crate::{
    gfx::{
        p_gltf::GltfManager,
        uniforms::{
            MaterialUniformContainer, ModelUniformContainer, NodeUniformContainer,
            SceneUniformContainer,
        },
        DeviceQueue, GfxSettings,
    },
    EventQueue,
};
use rendering_ir::{
    camera3d::Camera3d,
    wgpu_helpers::{create_render_pipeline, texture::Texture, vertex::Vertex},
};

// TODO: make this configurable + add rewriting of old assets.
const GLTF_SCENE_CAPACITY: usize = 200;

pub struct ScenePipeline {
    pipeline: wgpu::RenderPipeline,
    gfx_settings: GfxSettings,
    gltf_manager: GltfManager,
    camera: Camera3d,
    scene_ubo: SceneUniformContainer,
    node_ubo_layout: wgpu::BindGroupLayout,
    model_ubo_layout: wgpu::BindGroupLayout,
    material_ubo_layout: wgpu::BindGroupLayout,
}

impl ShaderPipeline for ScenePipeline {
    fn new(
        render_texture_format: Option<wgpu::TextureFormat>,
        gfx_settings: GfxSettings,
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let render_texture_format = match render_texture_format {
            Some(r) => r,
            None => {
                panic!("Required render texture format to be Some(wgpu::TextureFormat)!");
            }
        };

        let camera = {
            let (w, h) = gfx_settings.render_resolution;
            Camera3d::new(w as f32, h as f32, [0.0; 3], [0.0; 3], false)
        };

        let (scene_ubo_layout, scene_ubo) = {
            let scene_ubo_layout = SceneUniformContainer::init_bindgroup_layout(
                "scene_ubo",
                device,
                wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                None,
            );

            let ubo = uniforms::SceneUbo::new(&camera);
            let ubo =
                SceneUniformContainer::new(ubo, &scene_ubo_layout, &DeviceQueue { device, queue });
            (scene_ubo_layout, ubo)
        };

        let node_ubo_layout = NodeUniformContainer::init_bindgroup_layout(
            "node_ubo",
            device,
            wgpu::ShaderStage::VERTEX,
            None,
        );

        let model_ubo_layout = ModelUniformContainer::init_bindgroup_layout(
            "model_ubo",
            device,
            wgpu::ShaderStage::VERTEX,
            None,
        );

        let material_ubo_layout = MaterialUniformContainer::init_bindgroup_layout(
            "material_ubo",
            device,
            wgpu::ShaderStage::FRAGMENT,
            None,
        );

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &scene_ubo_layout,
                    &model_ubo_layout,
                    &node_ubo_layout,
                    &material_ubo_layout,
                ],
                push_constant_ranges: &[],
            });

        let pipeline = create_render_pipeline(
            &device,
            &render_pipeline_layout,
            Some("Render Pipeline"),
            render_texture_format,
            Some(Texture::DEPTH_FORMAT),
            &[p_gltf::Vertex::desc()],
            wgpu::include_spirv!("../../../gfx/shaders/spv/pbr.vert.spv"),
            wgpu::include_spirv!("../../../gfx/shaders/spv/pbr.frag.spv"),
        );

        Self {
            gfx_settings,
            pipeline,
            gltf_manager: GltfManager::new(GLTF_SCENE_CAPACITY),
            camera,
            // ubos
            scene_ubo,
            node_ubo_layout,
            model_ubo_layout,
            material_ubo_layout,
        }
    }

    fn update(
        &mut self,
        dq: &DeviceQueue,
        command_queue: &RenderQueue,
        event_queue: &mut EventQueue,
    ) {
        // Do any resets + processing here
        self.gltf_manager.process(
            &self.model_ubo_layout,
            &self.node_ubo_layout,
            &self.material_ubo_layout,
            dq,
            event_queue,
        );

        // Process commands
        for cmd in command_queue.commands() {
            match cmd {
                RenderCommand::CameraUpdate {
                    target,
                    eye,
                    orthographic,
                } => {
                    self.camera.update(*target, *eye, *orthographic);
                    self.scene_ubo.uniform_mut().update(&self.camera);
                }
                RenderCommand::Asset(asset_msg) => match asset_msg {
                    AssetCommand::LoadGltf { file } => {
                        self.gltf_manager.load_scene(file);
                    }
                    AssetCommand::DropGltf { file } => {
                        self.gltf_manager.drop_scene(file);
                    }
                    AssetCommand::LoadObj {
                        file,
                        max_instances,
                    } => {
                        println!("TODO: load obj file.");
                    }
                    AssetCommand::DropObj { file } => {
                        println!("TODO: drop obj file.");
                    }
                },
                RenderCommand::ModelDraw {
                    file,
                    position,
                    rotation,
                    scale,
                } => {
                    // Instancing
                }
                _ => {}
            }
        }

        // Update uniforms
        self.scene_ubo.write_buffer(dq);
    }

    fn render<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>) {
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

        render_pass.set_bind_group(0, self.scene_ubo.bind_group(), &[]);

        self.gltf_manager.render(render_pass);

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
}
