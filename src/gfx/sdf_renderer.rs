use futures::executor::block_on;
use winit::{event::*, window::Window};

use wgpu::util::DeviceExt;

use super::*;
use crate::event_queue::EventQueue;
use crate::lib_core::{
    ecs::World,
    time::{Clock, Duration},
};

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    // pipelines
    render_pipeline: wgpu::RenderPipeline,
    // camera
    camera: Camera,
    camera_controller: CameraController,
    // uniforms
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    //textures
    depth_texture: texture::Texture,
    voxel_uniform_buffer: wgpu::Buffer,
    voxel_uniform_bind_group: wgpu::BindGroup,
    //
    voxel_pass: voxels::texture_voxels::VoxelPass,
    //render timer
    clock: Clock,
    render_timer: Timer,
}

impl GfxRenderer for State {
    fn new(world: &World, window: &Window, fps: u32) -> Self {
        let mut size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: Some(&surface),
        }))
        .unwrap();
        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                shader_validation: true,
            },
            None, // Trace path
        ))
        .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate, //Fifo for mobile
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let camera = Camera {
            eye: (-10.0, 20.0, -20.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: sc_desc.width as f32 / sc_desc.height as f32,
            fovy: 45.0,
            znear: 0.01,
            zfar: 1000.0,
        };
        let camera_controller = CameraController::new(0.02);

        let mut uniforms = Uniforms::new(size.width as f32, size.height as f32);
        uniforms.update_view_proj(&camera);

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

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

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..)),
            }],
            label: Some("uniform_bind_group"),
        });

        let voxel_pass = voxels::texture_voxels::VoxelPass::new(&world, &device, &queue);

        let voxel_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Voxel Uniform Buffer"),
            contents: bytemuck::cast_slice(&[voxel_pass.voxel_uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let voxel_uniform_bind_group_layout =
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
                label: Some("voxel_uniform_bind_group_layout"),
            });

        let voxel_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &voxel_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(voxel_uniform_buffer.slice(..)),
            }],
            label: Some("voxel_uniform_bind_group"),
        });

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &sc_desc, "depth_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &uniform_bind_group_layout,
                    &voxel_uniform_bind_group_layout,
                    &voxel_pass.volume_tex.texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        use crate::gfx::vertex::Vertex;
        let render_pipeline = create_render_pipeline(
            &device,
            &render_pipeline_layout,
            Some("Render Pipeline"),
            sc_desc.format,
            Some(texture::Texture::DEPTH_FORMAT),
            &[],
            wgpu::include_spirv!("../gfx/shaders/sdf.vert.spv"),
            wgpu::include_spirv!("../gfx/shaders/sdf.frag.spv"),
        );

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            // pipelines
            render_pipeline,
            // camera
            camera,
            camera_controller,
            // uniforms
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            // textures
            depth_texture,
            //
            voxel_pass,
            voxel_uniform_buffer,
            voxel_uniform_bind_group,
            //
            clock: Clock::new(),
            render_timer: Timer::new(fps),
        }
    }
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.depth_texture =
            texture::Texture::create_depth_texture(&self.device, &self.sc_desc, "depth_texture");

        self.uniforms
            .update_viewport_size(new_size.width as f32, new_size.height as f32);
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    fn update(&mut self, world: &World) {
        self.camera_controller.update_camera(&mut self.camera);
        self.uniforms.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );

        self.voxel_pass.update(world, &self.device, &self.queue);
        // TODO: update
        self.queue.write_buffer(
            &self.voxel_uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.voxel_pass.voxel_uniforms]),
        );
    }

    fn render(&mut self) {
        let frame = self
            .swap_chain
            .get_current_frame()
            .expect("Timeout getting texture")
            .output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            // voxels
            render_pass.set_bind_group(1, &self.voxel_uniform_bind_group, &[]);
            render_pass.set_bind_group(2, &self.voxel_pass.volume_tex.bind_group, &[]);

            render_pass.draw(0..6, 0..1); // Draw a quad that takes the whole screen up
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        let duration = self.clock.stop_watch();
        println!("Frame time: {:?}", duration);
    }
    fn delta_time(&self) -> Duration {
        unimplemented!();
    }
    fn timer(&mut self) -> &mut Timer {
        &mut self.render_timer
    }
}
