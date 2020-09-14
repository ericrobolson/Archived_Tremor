pub mod client;
use client::Client;

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
    A central event queue is used for things.
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
    pub fn execute(&mut self, gfx: &mut OpenGlRenderer) -> Result<(), String> {
        //NOTE: the window has already written it's input so we can just proceed.
        self.socket_manager.read(&mut self.event_queue)?;
        self.journal.dump(&self.event_queue)?;
        self.server.execute(&self.event_queue)?;
        self.client.execute(&self.event_queue, gfx)?;
        self.event_queue.clear();

        Ok(())
    }
}

fn main() {
    let (mut event_loop, mut window_context, mut gfx) = window::build_window();

    let mut main_game = MainGame::new();

    event_loop.run(move |event, _, control_flow| {
        // Get events
        window::handle_event(event, &mut main_game.event_queue);

        match main_game.execute(&mut gfx) {
            Ok(()) => {}
            Err(e) => {
                println!("{}", e);
                loop {}
            }
        }
    });
}
