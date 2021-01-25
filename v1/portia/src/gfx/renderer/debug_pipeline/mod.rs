use super::{shader_pipeline::ShaderPipeline, *};
use crate::{
    gfx::{uniforms::SceneUniformContainer, DeviceQueue, GfxSettings},
    EventQueue,
};
use rendering_ir::{
    camera3d::Camera3d,
    wgpu_helpers::{
        create_render_pipeline,
        texture::Texture,
        vertex::{Index, Vertex},
    },
};

mod vertex;

const MAX_PRIMITIVES: usize = 400;

pub struct DebugPipeline {
    pipeline: wgpu::RenderPipeline,
    gfx_settings: GfxSettings,
    camera: Camera3d,
    scene_ubo: SceneUniformContainer,
    primitive_count: usize,
    index_len: u32,
    vert_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl ShaderPipeline for DebugPipeline {
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

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&scene_ubo_layout],
                push_constant_ranges: &[],
            });

        let pipeline = create_render_pipeline(
            &device,
            &render_pipeline_layout,
            Some("Render Pipeline"),
            render_texture_format,
            Some(Texture::DEPTH_FORMAT),
            &[vertex::Vertex::desc()],
            wgpu::include_spirv!("../../../gfx/shaders/spv/debug_scene.vert.spv"),
            wgpu::include_spirv!("../../../gfx/shaders/spv/debug_scene.frag.spv"),
        );

        // Prepopulate the buffers to a max range
        let (verts, indexes) = {
            let mut verts = vec![];
            let mut indexes = vec![];

            for _ in 0..MAX_PRIMITIVES {
                let (mut v, mut i) =
                    vertex::init_colored_quad(0.0, 0.0, 0.0, 0.0, 0.0, [0.0; 4], None);

                verts.append(&mut v);
                indexes.append(&mut i);
            }

            (verts, indexes)
        };

        let vert_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&indexes),
            usage: wgpu::BufferUsage::INDEX | wgpu::BufferUsage::COPY_DST,
        });

        Self {
            gfx_settings,
            pipeline,
            camera,
            // ubos
            scene_ubo,
            primitive_count: 0,
            index_len: 0,
            vert_buffer,
            index_buffer,
        }
    }
    fn update(
        &mut self,
        dq: &DeviceQueue,
        command_queue: &RenderQueue,
        event_queue: &mut EventQueue,
    ) {
        // Do any resets + processing here
        self.primitive_count = 0;
        self.index_len = 0;

        let mut verts = vec![];
        let mut indexes = vec![];

        let mut index_offset = 0;

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
                RenderCommand::DebugRectangle { min, max, z, color } => {
                    if self.primitive_count < MAX_PRIMITIVES {
                        self.primitive_count += 1;

                        let w = max[0] - min[0];
                        let h = max[1] - min[1];

                        let x = min[0];
                        let y = min[1] + h / 2.0;

                        let (mut v, mut i) =
                            vertex::init_colored_quad(x, y, *z, w, h, *color, Some(index_offset));

                        index_offset += v.len() as Index;

                        verts.append(&mut v);
                        indexes.append(&mut i);
                    }
                }
                _ => {}
            }
        }

        // Update buffers
        if self.primitive_count > 0 {
            self.index_len = indexes.len() as u32;

            // Write to buffers
            dq.queue
                .write_buffer(&self.vert_buffer, 0, bytemuck::cast_slice(&verts));

            dq.queue
                .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indexes));
        }

        // Update uniforms
        self.scene_ubo.write_buffer(dq);
    }

    fn render<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>) {
        if self.primitive_count == 0 {
            return;
        }

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

        render_pass.set_vertex_buffer(0, self.vert_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..));
        render_pass.draw_indexed(0..self.index_len, 0, 0..1);

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
