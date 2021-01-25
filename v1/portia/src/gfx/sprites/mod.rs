use rendering_ir::wgpu_helpers::{create_render_pipeline, texture, vertex::Vertex};

use crate::{gfx::DeviceQueue, gui::RenderCommand};

mod sprite_renderer;
use sprite_renderer::SpriteRenderer;
pub use sprite_renderer::TextRenderCommand;
mod vertex;

const FONT_CAPACITY: usize = 10;

pub struct SpritePipeline {
    sprites: Vec<SpriteRenderer>,
    tex_bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
    screen_size: (u32, u32),
}

impl SpritePipeline {
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
                    label: Some("Sprite Texture Layout"),
                });

        let render_pipeline = {
            let render_pipeline_layout =
                dq.device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Sprite Render Pipeline Layout"),
                        bind_group_layouts: &[&tex_bind_group_layout],
                        push_constant_ranges: &[],
                    });

            let render_pipeline = create_render_pipeline(
                &dq.device,
                &render_pipeline_layout,
                Some("Sprite Render Pipeline"),
                swap_chain_descriptor.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[vertex::SpriteVert::desc()],
                wgpu::include_spirv!("../shaders/spv/sprite.vert.spv"),
                wgpu::include_spirv!("../shaders/spv/sprite.frag.spv"),
            );

            render_pipeline
        };

        Self {
            sprites: Vec::with_capacity(FONT_CAPACITY),
            tex_bind_group_layout,
            render_pipeline,
            screen_size: (swap_chain_descriptor.width, swap_chain_descriptor.height),
        }
    }

    pub fn resize(&mut self, screen_width: u32, screen_height: u32) {
        self.screen_size = (screen_width, screen_height);
        for sprite_pipeline in self.sprites.iter_mut() {
            sprite_pipeline.resize(screen_width, screen_height);
        }
    }

    pub fn reset(&mut self) {
        for sprite_pipeline in self.sprites.iter_mut() {
            sprite_pipeline.reset();
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
        self.sprites.iter_mut().for_each(|f| f.execute(dq));
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);

        for sprite_pipeline in self.sprites.iter() {
            sprite_pipeline.render(render_pass);
        }
    }

    fn pipeline_index(&self, font: &'static str) -> Option<usize> {
        match self
            .sprites
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
        let matching_index = match self.pipeline_index(font) {
            Some(i) => i,
            None => {
                let index = self.sprites.len();
                self.sprites.push(SpriteRenderer::new(
                    font,
                    self.screen_size,
                    &self.tex_bind_group_layout,
                    dq,
                ));

                index
            }
        };

        self.sprites[matching_index].add_command(text_command);
    }
}
