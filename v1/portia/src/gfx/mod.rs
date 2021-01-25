use winit::{
    dpi::PhysicalPosition,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use std::sync::mpsc::{channel, Receiver, SendError, Sender};

use crate::{
    gui::{AssetCommand, RenderCommand, RenderQueue},
    input,
    time::{Clock, Timer},
    EventQueue,
};

mod fonts;
mod p_gltf;
mod renderer;
mod sprites;
mod uniform_container;
mod uniforms;

/// Behavior for double buffering two things.
pub enum DoubleBuffer {
    UpdateARenderB,
    UpdateBRenderA,
}

pub struct DeviceQueue<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
}

pub enum GfxMsgs {
    Render,
    Resize { width: u32, height: u32 },
}

pub struct InputHandler {
    cursor_position: (i32, i32),
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            cursor_position: (0, 0),
        }
    }

    pub fn handle_events<T>(
        &mut self,
        event: Event<T>,
        control_flow: &mut ControlFlow,
        window: &Window,
        render_sender: &Sender<GfxMsgs>,
    ) -> Option<input::Input> {
        match event {
            Event::RedrawRequested(_) => {
                match render_sender.send(GfxMsgs::Render) {
                    Ok(_) => {}
                    Err(_) => {}
                };
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } => {
                if window_id == window.id() {
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                            return Some(input::Input::ApplicationExit);
                        }
                        WindowEvent::Resized(physical_size) => {
                            match render_sender.send(GfxMsgs::Resize {
                                width: physical_size.width,
                                height: physical_size.height,
                            }) {
                                Ok(_) => {}
                                Err(e) => {
                                    use std::error::Error;
                                    println!("ERROR1: {:?}", e.description());
                                }
                            }

                            return Some(input::Input::Window(input::Window::Resized {
                                height: physical_size.height,
                                width: physical_size.width,
                            }));
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            match render_sender.send(GfxMsgs::Resize {
                                width: new_inner_size.width,
                                height: new_inner_size.height,
                            }) {
                                Ok(_) => {}
                                Err(e) => {
                                    println!("ERROR2: {:?}", e);
                                }
                            }

                            return Some(input::Input::Window(input::Window::Resized {
                                height: new_inner_size.height,
                                width: new_inner_size.width,
                            }));
                        }
                        WindowEvent::DroppedFile(path) => {
                            return Some(input::Input::Window(input::Window::DroppedFile(
                                path.to_path_buf(),
                            )));
                        }
                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state,
                                virtual_keycode,
                                ..
                            } => {
                                let input_state = match state {
                                    ElementState::Released => input::PressState::Released,
                                    ElementState::Pressed => input::PressState::Pressed,
                                };

                                if let Some(keycode) = virtual_keycode {
                                    match keycode {
                                        VirtualKeyCode::Left => {
                                            return Some(input::Input::Key {
                                                key: input::Key::Left,
                                                state: input_state,
                                            });
                                        }
                                        VirtualKeyCode::Up => {
                                            return Some(input::Input::Key {
                                                key: input::Key::Up,
                                                state: input_state,
                                            });
                                        }
                                        VirtualKeyCode::Right => {
                                            return Some(input::Input::Key {
                                                key: input::Key::Right,
                                                state: input_state,
                                            });
                                        }
                                        VirtualKeyCode::Down => {
                                            return Some(input::Input::Key {
                                                key: input::Key::Down,
                                                state: input_state,
                                            });
                                        }
                                        VirtualKeyCode::Space => {
                                            return Some(input::Input::Key {
                                                key: input::Key::Space,
                                                state: input_state,
                                            });
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        },
                        /*
                        WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                        } => *control_flow = ControlFlow::Exit,
                        KeyboardInput {
                        state, scancode, ..
                        } => {}
                        _ => {
                        //   println!("unhandled: {:?}", event);
                        }
                        },
                         */
                        WindowEvent::CursorMoved { position, .. } => {
                            let (x, y) = { (position.x, position.y) };

                            self.cursor_position = (x as i32, y as i32);

                            return Some(input::Input::Mouse(input::Mouse::Moved {
                                x: x as i32,
                                y: y as i32,
                            }));
                        }
                        WindowEvent::MouseInput { state, button, .. } => {
                            let is_pressed = match state {
                                ElementState::Pressed => true,
                                _ => false,
                            };

                            let button = match button {
                                MouseButton::Left => input::MouseButton::Left,
                                MouseButton::Right => input::MouseButton::Right,
                                MouseButton::Middle => input::MouseButton::Middle,
                                MouseButton::Other(b) => input::MouseButton::Other(*b),
                            };

                            let (x, y) = self.cursor_position;

                            let mouse_event = match is_pressed {
                                true => input::Mouse::Clicked { x, y, button },
                                false => input::Mouse::Released { x, y, button },
                            };

                            return Some(input::Input::Mouse(mouse_event));
                        }
                        _ => {
                            //  println!("unhandled: {:?}", event);
                        }
                    }
                } else {
                    //  println!("unhandled: {:?}", event);
                }
            }
            _ => {}
        }

        None
    }
}

pub trait GfxRenderer {
    fn new(window: &Window, settings: GfxSettings) -> Self;
    fn resize(&mut self, width: u32, height: u32);
    /// Queue up events for the renderer to process.
    fn update(&mut self, command_queue: &RenderQueue, event_queue: &mut EventQueue);
    /// Write the pixels to the screen.
    fn render(&mut self);
    fn timer(&mut self) -> &mut Timer;

    fn set_cursor_position(&mut self, x: i32, y: i32);
    fn cursor_position(&self) -> (i32, i32);
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GfxSettings {
    /// The physical size of the application
    pub physical_resolution: (u32, u32),
    /// The target render resolution, independent of hardware. TODO: ensure that when this is set, it's never larger than the physical resolution.
    pub render_resolution: (u32, u32),
    pub fps: u32,
}

pub fn setup(
    title: &'static str,
    settings: GfxSettings,
) -> (EventLoop<()>, Window, impl GfxRenderer) {
    let size = winit::dpi::PhysicalSize {
        width: settings.physical_resolution.0,
        height: settings.physical_resolution.1,
    };

    let min_size = winit::dpi::PhysicalSize {
        width: settings.render_resolution.0,
        height: settings.render_resolution.1,
    };

    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(title)
        .with_min_inner_size(min_size)
        .with_inner_size(size)
        .with_resizable(false)
        //.with_maximized(true)
        .build(&event_loop)
        .unwrap();

    let state = { renderer::State::new(&window, settings) };

    (event_loop, window, state)
}
