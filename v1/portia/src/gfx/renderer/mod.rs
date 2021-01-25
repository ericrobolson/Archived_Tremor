use super::*;
use rendering_ir::wgpu_helpers::texture::{Image, Texture};

use futures::executor::block_on;
use wgpu::util::DeviceExt;
use winit::window::Window;

mod debug_pipeline;
mod scene_pipeline;
mod scene_pipeline_obj;
mod shader_pipeline;
mod window_pipeline;

use shader_pipeline::{MetaShaderPipeline, ShaderPipeline};

pub struct State {
    cursor_position: (i32, i32),
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_swap_chain: SwapChainContainer,
    gfx_settings: GfxSettings,
    // pipelines
    window_renderer: MetaShaderPipeline<window_pipeline::WindowPipeline>,
    obj_scene_renderer: MetaShaderPipeline<scene_pipeline_obj::ScenePipeline>,
    scene_renderer: MetaShaderPipeline<scene_pipeline::ScenePipeline>,
    debug_renderer: MetaShaderPipeline<debug_pipeline::DebugPipeline>,
    //render timer
    clock: Clock,
    render_timer: Timer,
    // Sub engines
}

struct SwapChainContainer {
    swap_chain: wgpu::SwapChain,
    sc_desc: wgpu::SwapChainDescriptor,
    depth_texture: Texture,
}

impl GfxRenderer for State {
    fn new(window: &Window, settings: GfxSettings) -> Self {
        let physical_size = settings.physical_resolution;

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

        let render_swap_chain = {
            let window_sc_desc = wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width: physical_size.0,
                height: physical_size.1,
                present_mode: wgpu::PresentMode::Immediate, //Fifo for mobile
            };

            let window_swap_chain = device.create_swap_chain(&surface, &window_sc_desc);

            let depth_texture = Texture::create_depth_texture(
                &device,
                window_sc_desc.width,
                window_sc_desc.height,
                "window_depth_texture",
            );

            SwapChainContainer {
                sc_desc: window_sc_desc,
                swap_chain: window_swap_chain,
                depth_texture,
            }
        };

        let window_renderer = MetaShaderPipeline::<window_pipeline::WindowPipeline>::new(
            None,
            settings,
            &render_swap_chain.sc_desc,
            &device,
            &queue,
        );

        let obj_scene_renderer = MetaShaderPipeline::new(
            Some(window_renderer.shader().texture_format),
            settings,
            &render_swap_chain.sc_desc,
            &device,
            &queue,
        );

        let scene_renderer = MetaShaderPipeline::new(
            Some(window_renderer.shader().texture_format),
            settings,
            &render_swap_chain.sc_desc,
            &device,
            &queue,
        );

        let debug_renderer = MetaShaderPipeline::new(
            Some(window_renderer.shader().texture_format),
            settings,
            &render_swap_chain.sc_desc,
            &device,
            &queue,
        );

        Self {
            cursor_position: (0, 0),
            surface,
            device,
            queue,
            render_swap_chain,
            gfx_settings: settings,
            // pipelines
            window_renderer,
            obj_scene_renderer,
            scene_renderer,
            debug_renderer,
            //
            clock: Clock::new(),
            render_timer: Timer::new(settings.fps),
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.gfx_settings.physical_resolution = (width, height);

        self.render_swap_chain.sc_desc.width = width;
        self.render_swap_chain.sc_desc.height = height;
        self.render_swap_chain.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.render_swap_chain.sc_desc);

        {
            let dq = DeviceQueue {
                device: &self.device,
                queue: &self.queue,
            };

            self.window_renderer.initiate_resize(self.gfx_settings, &dq);
            self.scene_renderer.initiate_resize(self.gfx_settings, &dq);
            self.debug_renderer.initiate_resize(self.gfx_settings, &dq);
            self.obj_scene_renderer
                .initiate_resize(self.gfx_settings, &dq);
        }
        self.render_swap_chain.depth_texture =
            Texture::create_depth_texture(&self.device, width, height, "depth_texture");
    }
    fn update(&mut self, command_queue: &RenderQueue, event_queue: &mut EventQueue) {
        let dq = DeviceQueue {
            device: &self.device,
            queue: &self.queue,
        };

        self.window_renderer.update(&dq, command_queue, event_queue);
        self.scene_renderer.update(&dq, command_queue, event_queue);
        self.debug_renderer.update(&dq, command_queue, event_queue);
        self.obj_scene_renderer
            .update(&dq, command_queue, event_queue);
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

    fn render(&mut self) {
        // Most GPU uploads should occur at a fixed rate. To support faster hz, add in 'velocities' to all transforms and add in a delta T uniform value.
        // TODO: include a 'delta time' uniform and send that to shaders w/ velocities to integrate. That way faster refreshes are supported.

        let frame = self
            .render_swap_chain
            .swap_chain
            .get_current_frame()
            .expect("Timeout getting texture")
            .output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        // To start, render the scene to a texture. From that texture, scale it to fit the window.
        //https://github.com/gfx-rs/wgpu-rs/blob/master/examples/capture/main.rs
        let scene_texture = {
            let (w, h) = self.gfx_settings.physical_resolution;
            let texture_extent = wgpu::Extent3d {
                width: w,
                height: h,
                depth: 1,
            };

            // The render pipeline renders data into this texture
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                size: texture_extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.window_renderer.shader().texture_format,
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::COPY_SRC,
                label: None,
            });

            texture
        };
        {
            let scene_view = scene_texture.create_view(&wgpu::TextureViewDescriptor::default());
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &scene_view,
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
                    attachment: &self.render_swap_chain.depth_texture.view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            self.obj_scene_renderer.initiate_render(&mut render_pass);
            self.scene_renderer.initiate_render(&mut render_pass);
            self.debug_renderer.initiate_render(&mut render_pass);
        }
        {
            // Copy the rendered scene to the window texture
            encoder.copy_texture_to_texture(
                wgpu::TextureCopyView {
                    texture: &scene_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                wgpu::TextureCopyView {
                    texture: &self.window_renderer.shader().texture.texture(),
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                wgpu::Extent3d {
                    depth: 1,
                    width: self.gfx_settings.render_resolution.0,
                    height: self.gfx_settings.render_resolution.1,
                },
            );
        }
        {
            // Now that the scene has been rendered to the storage texture, render the scene on some triangles.
            let mut window_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.render_swap_chain.depth_texture.view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            self.window_renderer.initiate_render(&mut window_pass);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
    }
}
