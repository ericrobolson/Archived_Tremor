use crate::gfx::{DeviceQueue, GfxSettings, RenderQueue};
use crate::EventQueue;

/// How the highest layer of the renderer will operate the shader. Meant to automate pipeline management and make it more ergonomic to focus on specific shader implementations.
/// All functions except new() should provide a generic implementation.
pub struct MetaShaderPipeline<Shader>
where
    Shader: ShaderPipeline,
{
    shader: Shader,
}

impl<Shader> MetaShaderPipeline<Shader>
where
    Shader: ShaderPipeline,
{
    pub fn new(
        render_texture_format: Option<wgpu::TextureFormat>,
        gfx_settings: GfxSettings,
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        Self {
            shader: Shader::new(render_texture_format, gfx_settings, sc_desc, device, queue),
        }
    }

    /// Sets and renders the shader.
    pub fn initiate_render<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>) {
        render_pass.set_pipeline(&self.shader.pipeline());
        self.shader.render(render_pass);
    }

    pub fn shader(&self) -> &Shader {
        &self.shader
    }

    pub fn update(
        &mut self,
        dq: &DeviceQueue,
        command_queue: &RenderQueue,
        event_queue: &mut EventQueue,
    ) {
        self.shader.update(dq, command_queue, event_queue);
    }

    /// Initiate a resize for the system.
    pub fn initiate_resize(&mut self, new_settings: GfxSettings, dq: &DeviceQueue) {
        let old_settings = self.shader.gfx_settings();
        self.shader.resize(old_settings, new_settings, dq);
        self.shader.replace_gfx_settings(new_settings);
    }
}

/// The specific things each shader pipeline must fill.
pub trait ShaderPipeline {
    fn new(
        render_texture_format: Option<wgpu::TextureFormat>,
        gfx_settings: GfxSettings,
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self;

    /// Returns the min + max values for the viewport depth.
    fn viewport_depth_values() -> (f32, f32) {
        (0.0, 1.0)
    }

    /// The handle to the WGPU render pipeline.
    fn pipeline(&self) -> &wgpu::RenderPipeline;

    /// Reads from the RenderQueue, doing any processing + rendering that is required. May output to the event_queue.
    fn update(
        &mut self,
        dq: &DeviceQueue,
        command_queue: &RenderQueue,
        event_queue: &mut EventQueue,
    );

    /// Receives the current graphic settings.
    fn gfx_settings(&self) -> GfxSettings;
    /// Replaces the current graphic settings.
    fn replace_gfx_settings(&mut self, new_settings: GfxSettings);

    /// Things to process while resizing. Internal GfxSettings will be replaced after this call.
    fn resize(&mut self, old_settings: GfxSettings, new_settings: GfxSettings, dq: &DeviceQueue);

    /// The actual pipeline commands performed by the shader.
    fn render<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>);
}

// idea for how an actor would look
pub struct ActorManager<TMsg, TAddress>
where
    TAddress: PartialEq,
{
    actors: Vec<Box<dyn Actor<TAddress, Message = TMsg>>>,
    message_queue: Vec<(TAddress, TMsg)>,
}

impl<TMsg, TAddress> ActorManager<TMsg, TAddress>
where
    TAddress: PartialEq,
{
    fn execute(&mut self) {
        self.message_queue.clear();
        for actor in self.actors.iter_mut() {
            match actor.retrieve_outgoing_messages() {
                Some(outgoing) => {
                    self.message_queue.push(outgoing);
                }
                None => {}
            }
        }

        for (address, message) in &self.message_queue {
            for actor in self.actors.iter_mut() {
                if actor.address() == *address {
                    //   actor.recieve_message(*address, *message);
                }
            }
        }
    }
}

pub trait Actor<TAddress> {
    type Message;

    fn can_execute(&self) -> bool;
    fn address(&self) -> TAddress;

    fn recieve_message(&mut self, from_address: TAddress, input: Self::Message);
    fn retrieve_outgoing_messages(&mut self) -> Option<(TAddress, Self::Message)>;
}
