use crate::event_queue;
use event_queue::*;

use crate::event_journal;
use crate::gfx;
use crate::socket_manager;

use gfx::GfxVm;

use super::*;

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

pub struct Window {
    gfx_vm: GfxVm,
    gfx: OpenGlRenderer,
    input_converter: InputConverter,
    window_context: glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>,
}

impl Window {
    pub fn new() -> (Self, glutin::event_loop::EventLoop<()>) {
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

        (
            Self {
                gfx_vm: GfxVm::new(),
                gfx: gfx,
                window_context: windowed_context,
                input_converter: InputConverter::new(),
            },
            el,
        )
    }

    pub fn translate_event(
        &mut self,
        event: glutin::event::Event<()>,
        event_queue: &mut EventQueue,
    ) -> Result<(), String> {
        match event {
            Event::WindowEvent { event, .. } => match event {
                /*
                Cursor
                */
                WindowEvent::CursorMoved {
                    device_id: _,
                    position,
                    modifiers,
                } => {
                    let x = position.x;
                    let y = position.y;
                }
                /*
                Mouse
                */
                WindowEvent::MouseInput {
                    device_id: _,
                    state,
                    button,
                    modifiers,
                } => {
                    let pressed = match state {
                        glutin::event::ElementState::Pressed => event_queue::ButtonState::Pressed,
                        _ => event_queue::ButtonState::Released,
                    };

                    match button {
                        glutin::event::MouseButton::Left => {
                            event_queue.add(Events::Mouse(MouseEvents::LeftButton(pressed)))?;
                        }
                        glutin::event::MouseButton::Right => {
                            event_queue.add(Events::Mouse(MouseEvents::RightButton(pressed)))?;
                        }
                        glutin::event::MouseButton::Middle => {
                            event_queue.add(Events::Mouse(MouseEvents::MiddleButton(pressed)))?;
                        }
                        glutin::event::MouseButton::Other(button) => {
                            event_queue
                                .add(Events::Mouse(MouseEvents::OtherButton(button, pressed)))?;
                        }
                    }
                }
                /*
                Keyboard
                */
                WindowEvent::KeyboardInput {
                    device_id,
                    input,
                    is_synthetic: is_synthetic,
                } => match self.input_converter.match_keycode(input) {
                    Some(e) => {
                        event_queue.add(e)?;
                    }
                    None => {}
                },
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }

    pub fn render(&mut self) {
        // TODO: move to vm?
        self.gfx.render();
        self.window_context.swap_buffers().unwrap();
    }
}

struct InputConverter {
    held_keys: [Option<u32>; Self::HELD_KEYS_SIZE],
}

impl InputConverter {
    const HELD_KEYS_SIZE: usize = 16;

    pub fn new() -> Self {
        Self {
            held_keys: [None; Self::HELD_KEYS_SIZE],
        }
    }

    pub fn match_keycode(&mut self, event: glutin::event::KeyboardInput) -> Option<Events> {
        match event.state {
            glutin::event::ElementState::Pressed => {
                // If key is held, do nothing
                for i in 0..Self::HELD_KEYS_SIZE {
                    if self.held_keys[i].is_some() {
                        if self.held_keys[i].unwrap() == event.scancode {
                            return None;
                        }
                    }
                }
                // Else, we'll add it from scratch
                for i in 0..Self::HELD_KEYS_SIZE {
                    if self.held_keys[i].is_none() {
                        self.held_keys[i] = Some(event.scancode);

                        return Some(Events::Keyboard {
                            pressed: event_queue::ButtonState::Pressed,
                            scancode: event.scancode,
                        });
                    }
                }
            }
            glutin::event::ElementState::Released => {
                // remove key
                for i in 0..Self::HELD_KEYS_SIZE {
                    if self.held_keys[i].is_some() {
                        if self.held_keys[i].unwrap() == event.scancode {
                            self.held_keys[i] = None;

                            return Some(Events::Keyboard {
                                pressed: event_queue::ButtonState::Released,
                                scancode: event.scancode,
                            });
                        }
                    }
                }
            }
        }

        None
    }
}
