use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub mod camera;
pub mod texture;
use camera::Camera;
pub mod uniforms;
use uniforms::Uniforms;
pub mod poly_renderer;
pub mod sdf_renderer;
pub mod vertex;
pub mod voxels;
use voxels::VoxelChunkVertex;
pub mod conversions;
pub mod model_transform;

use crate::event_queue::EventQueue;
use crate::lib_core::{
    ecs::World,
    time::{Duration, Timer},
};

pub trait GfxRenderer {
    fn new(world: &World, window: &Window, fps: u32) -> Self;
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);
    fn input(&mut self, event: &WindowEvent) -> bool {
        // TODO: wire up input reading here. Return true if it was a handled game input.
        false
    }
    fn update(&mut self, world: &World);
    fn render(&mut self);
    fn delta_time(&self) -> Duration;
    fn timer(&mut self) -> &mut Timer;

    fn handle_events<T>(
        &mut self,
        event: Event<T>,
        control_flow: &mut ControlFlow,
        window: &Window,
        event_queue: &mut EventQueue,
    ) {
        match event {
            Event::RedrawRequested(_) => {
                self.render();
            }
            Event::MainEventsCleared => {
                if self.timer().can_run() {
                    window.request_redraw();
                }
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !self.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            self.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            self.resize(**new_inner_size);
                        }
                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn setup(world: &World, fps: u32) -> (EventLoop<()>, Window, impl GfxRenderer) {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_maximized(true)
        .build(&event_loop)
        .unwrap();

    let state = {
        poly_renderer::State::new(world, &window, fps)
        //sdf_renderer::State::new(world, &window, fps) // Note: think there's an issue with this. May be confusing certain axises as it looks taller than it should be. Maybe z and y got mixed?
    };

    (event_loop, window, state)
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    label: Option<&str>,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_descs: &[wgpu::VertexBufferDescriptor],
    vert_src: wgpu::ShaderModuleSource,
    frag_src: wgpu::ShaderModuleSource,
) -> wgpu::RenderPipeline {
    let vs_module = device.create_shader_module(vert_src);
    let fs_module = device.create_shader_module(frag_src);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: label,
        layout: Some(&layout),
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: &fs_module,
            entry_point: "main",
        }),
        rasterization_state: Some(wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::Back,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
            clamp_depth: false,
        }),
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[wgpu::ColorStateDescriptor {
            format: color_format,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }],
        depth_stencil_state: depth_format.map(|format| wgpu::DepthStencilStateDescriptor {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilStateDescriptor::default(),
        }),
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint32,
            vertex_buffers: vertex_descs,
        },
    })
}
