use rendering_ir::wgpu_helpers::{create_render_pipeline, texture::Texture, vertex::Vertex};

use crate::{gfx::DeviceQueue, gui::RenderCommand};

mod font_renderer;
use font_renderer::FontRenderer;
pub use font_renderer::TextRenderCommand;
mod vertex;

const FONT_CAPACITY: usize = 10;

pub struct FontPipeline {
    fonts: Vec<FontRenderer>,
    tex_bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
    screen_size: (u32, u32),
}

impl FontPipeline {
    pub fn new(dq: &DeviceQueue, swap_chain_descriptor: &wgpu::SwapChainDescriptor) -> Self {
        let tex_bind_group_layout =
            dq.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
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
                    ],
                    label: Some("Font Texture Layout"),
                });

        let render_pipeline = {
            let render_pipeline_layout =
                dq.device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Font Render Pipeline Layout"),
                        bind_group_layouts: &[&tex_bind_group_layout],
                        push_constant_ranges: &[],
                    });

            let render_pipeline = create_render_pipeline(
                &dq.device,
                &render_pipeline_layout,
                Some("Font Render Pipeline"),
                swap_chain_descriptor.format,
                Some(Texture::DEPTH_FORMAT),
                &[vertex::FontVert::desc()],
                wgpu::include_spirv!("../shaders/spv/text.vert.spv"),
                wgpu::include_spirv!("../shaders/spv/text.frag.spv"),
            );

            render_pipeline
        };

        Self {
            fonts: Vec::with_capacity(FONT_CAPACITY),
            tex_bind_group_layout,
            render_pipeline,
            screen_size: (swap_chain_descriptor.width, swap_chain_descriptor.height),
        }
    }

    pub fn resize(&mut self, screen_width: u32, screen_height: u32) {
        self.screen_size = (screen_width, screen_height);
        for font in self.fonts.iter_mut() {
            font.resize(screen_width, screen_height);
        }
    }

    pub fn reset(&mut self) {
        for font in self.fonts.iter_mut() {
            font.reset();
        }
    }

    pub fn execute(&mut self, commands: &Vec<RenderCommand>, dq: &DeviceQueue) {
        // Batch all commands
        for command in commands {
            match command {
                RenderCommand::Text {
                    position,
                    text,
                    font,
                    font_size,
                    color,
                } => {
                    self.process_text(
                        font,
                        TextRenderCommand {
                            position: *position,
                            text: text.clone(),
                            font_size: *font_size,
                            color: *color,
                        },
                        dq,
                    );
                }
                _ => {}
            }
        }

        // Now execute the generation of verts + indexes for each font and buffer the data.
        self.fonts.iter_mut().for_each(|f| f.execute(dq));
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);

        for font in self.fonts.iter() {
            font.render(render_pass);
        }
    }

    fn font_index(&self, font: &'static str) -> Option<usize> {
        match self
            .fonts
            .iter()
            .enumerate()
            .filter(|(i, f)| f.name == font)
            .peekable()
            .peek()
        {
            Some((i, _)) => Some(*i),
            None => None,
        }
    }

    fn process_text(
        &mut self,
        font: &'static str,
        text_command: TextRenderCommand,
        dq: &DeviceQueue,
    ) {
        // Ensure font is created
        let matching_index = match self.font_index(font) {
            Some(i) => i,
            None => {
                let index = self.fonts.len();
                self.fonts.push(FontRenderer::new(
                    font,
                    self.screen_size,
                    &self.tex_bind_group_layout,
                    dq,
                ));

                index
            }
        };

        self.fonts[matching_index].add_command(text_command);
    }
}
