pub mod client;
use client::Client;

pub mod lib_core;
use lib_core::LookUpGod;

pub mod event_journal;
use event_journal::EventJournal;

pub mod event_queue;
use event_queue::*;

pub mod gfx;
use gfx::OpenGlRenderer;

pub mod network;

pub mod server;
use server::Server;

pub mod socket_manager;
use socket_manager::SocketManager;

pub mod constants;
pub mod logging;

mod window;

/*
    This engine follows the model of Quake 3 (https://fabiensanglard.net/quake3/).
    A central event queue is used for communicating between systems.
*/

pub struct MainGame {
    lug: LookUpGod,
    client: Client,
    server: Server,
    client_input_handler: lib_core::input::ClientInputMapper,
    journal: EventJournal,
    socket_manager: SocketManager,
    pub event_queue: EventQueue,
    pub socket_out_event_queue: EventQueue,
}

impl MainGame {
    pub fn new() -> Result<Self, String> {
        let socket_manager = SocketManager::new("127.0.0.1:3400", "127.0.0.1:3401")?;
        Ok(Self {
            lug: LookUpGod::new(),
            client: Client::new(),
            server: Server::new(),
            client_input_handler: lib_core::input::ClientInputMapper::new(),
            socket_manager: socket_manager,
            journal: EventJournal::new(),
            event_queue: EventQueue::new(),
            socket_out_event_queue: EventQueue::new(),
        })
    }
    pub fn execute(&mut self) -> Result<(), String> {
        //NOTE: the window has already written it's input so we can just proceed.

        // Input mapper
        {
            self.client_input_handler.execute(&mut self.event_queue)?;
        }

        // Sockets
        {
            self.socket_manager.poll(
                &self.lug,
                &mut self.event_queue,
                &self.socket_out_event_queue,
            )?;
            // Clear the queue out so we can write client/server messages
            self.socket_out_event_queue.clear();
        }

        // Dump inputs + execute sims
        {
            self.journal.dump(&self.event_queue)?;
            self.server
                .execute(&self.event_queue, &mut self.socket_out_event_queue)?;
            self.client
                .execute(&self.event_queue, &mut self.socket_out_event_queue)?;
        }

        // Clear the event queue so we can start fresh next process
        self.event_queue.clear();

        Ok(())
    }
}

fn main() {
    let (mut window, event_loop) = window::Window::new();

    let mut main_game = match MainGame::new() {
        Ok(mg) => mg,
        Err(e) => {
            println!("{}", e);
            loop {}
        }
    };

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
