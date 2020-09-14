use crate::event_queue;
use event_queue::*;

use crate::event_journal;
use crate::gfx;
use crate::socket_manager;

use super::*;

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

pub fn build_window() -> (
    glutin::event_loop::EventLoop<()>,
    glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>,
    gfx::OpenGlRenderer,
) {
    let el = EventLoop::new();
    let wb = WindowBuilder::new().with_title("Tremor");

    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let window_size = windowed_context.window().inner_size();
    let mut gfx = gfx::OpenGlRenderer::new(
        &windowed_context.context(),
        window_size.height,
        window_size.width,
    );

    (el, windowed_context, gfx)
}

pub fn handle_event(event: glutin::event::Event<()>, event_queue: &mut EventQueue) {}
