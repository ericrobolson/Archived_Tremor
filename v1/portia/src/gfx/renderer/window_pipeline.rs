use super::{shader_pipeline::ShaderPipeline, *};
use crate::EventQueue;
use rendering_ir::wgpu_helpers::{
    create_render_pipeline,
    texture::{Image, Texture, TextureAtlas},
    vertex::{textured_indexed_quad, Index, TextureRegion, Vertex},
};

const TEXTURE_BINDGROUP: u32 = 0;
pub struct WindowPipeline {
    pub texture: TextureAtlas,
    pub tex_bind_group: wgpu::BindGroup,
    pub tex_bind_group_layout: wgpu::BindGroupLayout,
    pub texture_format: wgpu::TextureFormat,

    vert_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_len: u32,
    pipeline: wgpu::RenderPipeline,
    gfx_settings: GfxSettings,
}

fn window_texture(
    format: wgpu::TextureFormat,
    tex_bind_group_layout: &wgpu::BindGroupLayout,
    gfx_settings: &GfxSettings,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> (TextureAtlas, wgpu::BindGroup) {
    let texture = match TextureAtlas::copy_texture(
        device,
        queue,
        gfx_settings.render_resolution.0,
        gfx_settings.render_resolution.1,
        None,
        false,
        false,
        format,
    ) {
        Ok(texture) => texture,
        Err(e) => {
            panic!("{:?}", e);
        }
    };

    let tex_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &tex_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture.view()),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&texture.sampler()),
            },
        ],
        label: Some("window texture"),
    });

    (texture, tex_bind_group)
}

impl ShaderPipeline for WindowPipeline {
    fn new(
        _: Option<wgpu::TextureFormat>,
        gfx_settings: GfxSettings,
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let texture_format = wgpu::TextureFormat::Rgba8UnormSrgb;

        let storage_texture_layout = {
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::StorageTexture {
                    readonly: false,
                    format: sc_desc.format,
                    dimension: wgpu::TextureViewDimension::D2,
                    // component_type: wgpu::TextureComponentType::Uint,
                },
                count: None,
            }
        };

        let tex_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    storage_texture_layout.clone(),
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                        count: None,
                    },
                ],
                label: Some("Window Texture Layout"),
            });

        let (texture, tex_bind_group) = window_texture(
            texture_format,
            &tex_bind_group_layout,
            &gfx_settings,
            device,
            queue,
        );

        let (verts, indexes) = init_vert_indexes(&gfx_settings);
        let index_len = indexes.len() as u32;

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

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Window Pipeline Layout"),
                bind_group_layouts: &[&tex_bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = create_render_pipeline(
            &device,
            &render_pipeline_layout,
            Some("Render Pipeline"),
            sc_desc.format,
            Some(Texture::DEPTH_FORMAT),
            &[WindowVert::desc()],
            wgpu::include_spirv!("../../gfx/shaders/spv/window.vert.spv"),
            wgpu::include_spirv!("../../gfx/shaders/spv/window.frag.spv"),
        );

        Self {
            gfx_settings,
            texture,
            tex_bind_group,
            tex_bind_group_layout,
            vert_buffer,
            index_buffer,
            index_len,
            pipeline,
            texture_format,
        }
    }

    fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    fn update(
        &mut self,
        dq: &DeviceQueue,
        command_queue: &RenderQueue,
        event_queue: &mut EventQueue,
    ) {
    }

    fn gfx_settings(&self) -> GfxSettings {
        self.gfx_settings
    }

    fn replace_gfx_settings(&mut self, new_settings: GfxSettings) {
        self.gfx_settings = new_settings;
    }

    fn render<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>) {
        render_pass.set_pipeline(&self.pipeline);
        let window_res = self.gfx_settings.physical_resolution;
        let tex_bind_group = &self.tex_bind_group;
        let vert_buf = &self.vert_buffer;
        let index_buf = &self.index_buffer;
        let index_len = self.index_len;

        render_pass.set_bind_group(TEXTURE_BINDGROUP, tex_bind_group, &[]);
        render_pass.set_vertex_buffer(0, vert_buf.slice(..));
        render_pass.set_index_buffer(index_buf.slice(..));
        render_pass.draw_indexed(0..index_len, 0, 0..1);
    }

    fn resize(&mut self, old_settings: GfxSettings, new_settings: GfxSettings, dq: &DeviceQueue) {
        self.gfx_settings = new_settings;

        /*
        // If need to recreate texture:
        let (texture, tex_bind_group) = window_texture(
            &self.tex_bind_group_layout,
            &self.gfx_settings,
            dq.device,
            dq.queue,
        );

        self.texture = texture;
        self.tex_bind_group = tex_bind_group;
        */
        let (verts, indexes) = init_vert_indexes(&self.gfx_settings);

        dq.queue
            .write_buffer(&self.vert_buffer, 0, bytemuck::cast_slice(&verts));

        dq.queue
            .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indexes));
    }
}

impl WindowPipeline {}

fn init_vert_indexes(gfx_settings: &GfxSettings) -> (Vec<WindowVert>, Vec<Index>) {
    // Now scale the inner window to match the outer one
    let w_res = gfx_settings.physical_resolution;
    let r_res = gfx_settings.render_resolution;

    let (x, y, w, h) = {
        // Initial values to take up entire screen
        let mut x = -1.;
        let mut y = 0.;
        let mut w = 2.;
        let mut h = 2.;

        let (scalew, scaleh) = aspect_ratio_scale(
            r_res.0 as f32,
            r_res.1 as f32,
            w_res.0 as f32,
            w_res.1 as f32,
        );

        if scalew < scaleh {
            // center x
            x += 1. - scalew;
        } else {
            // adjust y
            y += 1. - scaleh;
        }

        w *= scalew;
        h *= scaleh;

        (x, y, w, h)
    };

    let (verts, indexes) = textured_indexed_quad(
        x,
        y,
        w,
        h,
        Some(TextureRegion {
            min_x: 0.0,
            min_y: 0.0,
            max_x: r_res.0 as f32,
            max_y: r_res.1 as f32,
        }),
        None,
    );

    let verts: Vec<WindowVert> = verts
        .iter()
        .map(|(v, tex)| WindowVert {
            position: [v[0], v[1]],
            texture_coords: *tex,
        })
        .collect();

    (verts, indexes)
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct WindowVert {
    pub position: [f32; 2],
    pub texture_coords: [f32; 2],
}

unsafe impl bytemuck::Pod for WindowVert {}
unsafe impl bytemuck::Zeroable for WindowVert {}

impl Vertex for WindowVert {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<WindowVert>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
            ],
        }
    }
}

fn aspect_ratio_scale(wscale: f32, hscale: f32, width: f32, height: f32) -> (f32, f32) {
    let w_ratio = width / wscale;
    let h_ratio = height / hscale;

    if w_ratio > h_ratio {
        let t = (wscale / hscale) * (height / width);

        return (t, 1.);
    } else {
        let v = (width / height) * (hscale / wscale);
        return (1., v);
    }

    (1., 1.)
}

#[cfg(test)]
mod gfx_tests {
    use super::*;

    #[test]
    fn aspect_ratio_scale_same_ratio_returns_1_1() {
        let wscale = 16.0;
        let hscale = 9.0;

        let width = 640.0;
        let height = 360.0;

        let (scale_x, scale_y) = aspect_ratio_scale(wscale, hscale, width, height);
        let expected_scale_x = 1.0;
        let expected_scale_y = 1.0;

        assert_eq!(expected_scale_x, scale_x);
        assert_eq!(expected_scale_y, scale_y);
    }

    #[test]
    fn aspect_ratio_scale_double_width_returns_expected() {
        let wscale = 16.0;
        let hscale = 9.0;

        let width = 640.0 * 2.;
        let height = 360.0;

        let (scale_x, scale_y) = aspect_ratio_scale(wscale, hscale, width, height);
        let expected_scale_x = 0.5;
        let expected_scale_y = 1.0;

        assert_eq!(expected_scale_x, scale_x);
        assert_eq!(expected_scale_y, scale_y);
    }

    #[test]
    fn aspect_ratio_scale_quadruple_width_returns_expected() {
        let wscale = 16.0;
        let hscale = 9.0;

        let width = 640.0 * 4.;
        let height = 360.0;

        let (scale_x, scale_y) = aspect_ratio_scale(wscale, hscale, width, height);
        let expected_scale_x = 0.25;
        let expected_scale_y = 1.0;

        assert_eq!(expected_scale_x, scale_x);
        assert_eq!(expected_scale_y, scale_y);
    }

    #[test]
    fn aspect_ratio_scale_double_height_returns_expected() {
        let wscale = 16.0;
        let hscale = 9.0;

        let width = 640.0;
        let height = 360.0 * 2.;

        let (scale_x, scale_y) = aspect_ratio_scale(wscale, hscale, width, height);
        let expected_scale_x = 1.;
        let expected_scale_y = 0.5;

        assert_eq!(expected_scale_x, scale_x);
        assert_eq!(expected_scale_y, scale_y);
    }

    #[test]
    fn aspect_ratio_scale_quadruple_height_returns_expected() {
        let wscale = 16.0;
        let hscale = 9.0;

        let width = 640.0;
        let height = 360.0 * 4.;

        let (scale_x, scale_y) = aspect_ratio_scale(wscale, hscale, width, height);
        let expected_scale_x = 1.;
        let expected_scale_y = 0.25;

        assert_eq!(expected_scale_x, scale_x);
        assert_eq!(expected_scale_y, scale_y);
    }
}
