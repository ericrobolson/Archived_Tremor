use super::*;
use super::{
    camera::Camera,
    texture::{Image, Texture},
    uniforms::Uniforms,
};

use futures::executor::block_on;
use wgpu::util::DeviceExt;
use winit::window::Window;

pub struct BindGroups {
    // uniforms
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}
impl BindGroups {
    pub const UNIFORMS: u32 = 0;
}

pub struct State {
    cursor_position: (i32, i32),
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    gfx_settings: GfxSettings,
    // pipelines
    render_pipeline: wgpu::RenderPipeline,
    // camera
    camera: camera::Camera,
    // uniforms
    uniforms: Uniforms,
    //textures
    depth_texture: texture::Texture,
    // bind groups
    bind_groups: BindGroups,
    //render timer
    clock: Clock,
    render_timer: Timer,
    // Sub engines
    font_pipeline: fonts::FontPipeline,
}

impl GfxRenderer for State {
    fn new(window: &Window, settings: GfxSettings) -> Self {
        let size = settings.render_resolution;

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
            width: size.0,
            height: size.1,
            present_mode: wgpu::PresentMode::Immediate, //Fifo for mobile
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let camera = camera::Camera::default(sc_desc.width as f32, sc_desc.height as f32);

        let mut uniforms = Uniforms::new(size.0 as f32, size.1 as f32);
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

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &sc_desc, "depth_texture");

        let bind_groups = BindGroups {
            uniform_buffer,
            uniform_bind_group,
        };

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = create_render_pipeline(
            &device,
            &render_pipeline_layout,
            Some("Render Pipeline"),
            sc_desc.format,
            Some(texture::Texture::DEPTH_FORMAT),
            &[],
            wgpu::include_spirv!("../gfx/shaders/spv/sprite.vert.spv"),
            wgpu::include_spirv!("../gfx/shaders/spv/sprite.frag.spv"),
        );

        let font_pipeline = {
            fonts::FontPipeline::new(
                &DeviceQueue {
                    device: &device,
                    queue: &queue,
                },
                &sc_desc,
            )
        };

        Self {
            cursor_position: (0, 0),
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            gfx_settings: settings,
            // pipelines
            render_pipeline,
            // camera
            camera,
            // uniforms
            uniforms,
            //
            bind_groups,
            // textures
            depth_texture,
            //
            clock: Clock::new(),
            render_timer: Timer::new(settings.fps),
            // Subengines
            font_pipeline,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.depth_texture =
            texture::Texture::create_depth_texture(&self.device, &self.sc_desc, "depth_texture");

        self.camera
            .resize(new_size.width as f32, new_size.height as f32);

        self.uniforms
            .update_viewport_size(new_size.width as f32, new_size.height as f32);

        self.font_pipeline.resize(new_size.width, new_size.height);
    }
    fn update(&mut self, stack: &RenderStack) {
        //self.camera.update(world);

        {
            let dq = DeviceQueue {
                device: &self.device,
                queue: &self.queue,
            };
            self.font_pipeline.reset();
            self.font_pipeline.execute(&stack.commands(), &dq);
        }

        self.uniforms.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.bind_groups.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }
    fn render(&mut self) {
        // Most GPU uploads should occur at a fixed rate. To support faster hz, add in 'velocities' to all transforms and add in a delta T uniform value.
        // TODO: include a 'delta time' uniform and send that to shaders w/ velocities to integrate. That way faster refreshes are supported.

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
                    attachment: &self.depth_texture.view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            self.font_pipeline.render(&mut render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        println!("Frame time: {:?}", self.clock.stop_watch());
    }
    fn timer(&mut self) -> &mut Timer {
        &mut self.render_timer
    }

    fn set_cursor_position(&mut self, x: i32, y: i32) {
        self.cursor_position = (x, y);
    }

    fn cursor_position(&self) -> (i32, i32) {
        self.cursor_position
    }
}
