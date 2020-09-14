pub mod client;
use client::Client;

pub mod lib_core;

pub mod event_journal;
use event_journal::EventJournal;

pub mod event_queue;
use event_queue::*;

pub mod gfx;
use gfx::OpenGlRenderer;

pub mod server;
use server::Server;

pub mod socket_manager;
use socket_manager::SocketManager;

mod window;

/*
    This engine follows the model of Quake 3 (https://fabiensanglard.net/quake3/).
    A central event queue is used for communicating between systems.
*/

pub struct MainGame {
    client: Client,
    server: Server,
    journal: EventJournal,
    pub event_queue: EventQueue,
    socket_manager: SocketManager,
}

impl MainGame {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            server: Server::new(),
            journal: EventJournal::new(),
            event_queue: EventQueue::new(),
            socket_manager: SocketManager::new(),
        }
    }
    pub fn execute(&mut self) -> Result<(), String> {
        //NOTE: the window has already written it's input so we can just proceed.
        self.socket_manager.read(&mut self.event_queue)?;
        self.journal.dump(&self.event_queue)?;
        self.server.execute(&self.event_queue)?;
        self.client.execute(&self.event_queue)?;
        self.event_queue.clear();

        Ok(())
    }
}

fn main() {
    let (mut window, mut event_loop) = window::Window::new();

    let mut main_game = MainGame::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = glutin::event_loop::ControlFlow::Poll;

        window
            .translate_event(event, &mut main_game.event_queue)
            .unwrap();

        match main_game.execute() {
            Ok(()) => {
                window.render();
            }
            Err(e) => {
                println!("{}", e);
                loop {}
            }
        }
    });
}
