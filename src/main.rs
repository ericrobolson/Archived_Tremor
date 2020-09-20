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
use network::{
    connection_layer::ConnectionManager, socket_manager::SocketManager,
    stream_manager::StreamManager,
};

pub mod server;
use server::Server;

pub mod constants;
pub mod logging;

pub mod window;
use window::WindowRenderer;

/*
    This engine follows the model of Quake 3 (https://fabiensanglard.net/quake3/).
    A central event queue is used for communicating between systems.
*/

pub struct MainGame {
    lug: LookUpGod,
    client: Client,
    journal: EventJournal,
    socket_manager: SocketManager,
    window_renderer: WindowRenderer,
    pub event_queue: EventQueue,
    pub socket_out_event_queue: EventQueue,
}

impl MainGame {
    pub fn new(window_renderer: WindowRenderer) -> Result<Self, String> {
        let socket_manager = SocketManager::new("127.0.0.1:3400")?;
        Ok(Self {
            lug: LookUpGod::new(),
            client: Client::new(),
            socket_manager: socket_manager,
            window_renderer: window_renderer,
            journal: EventJournal::new(),
            event_queue: EventQueue::new(),
            socket_out_event_queue: EventQueue::new(),
        })
    }
    pub fn execute(&mut self) -> Result<(), String> {
        //NOTE: the window has already written it's input so we can just proceed.

        // Network
        {
            self.socket_manager.poll(
                &self.lug,
                &mut self.event_queue,
                &self.socket_out_event_queue,
            )?;

            // Clear the queue out
            self.socket_out_event_queue.clear();
        }

        // Dump inputs + execute sims
        {
            self.journal.dump(&self.event_queue)?;

            self.client.execute(
                &mut self.event_queue,
                &mut self.socket_out_event_queue,
                &mut self.window_renderer,
            )?;
        }

        // Clear the event queue so we can start fresh next process
        self.event_queue.clear();

        Ok(())
    }
}

fn main() {
    //TODO: Start as CLI to run server/client?
    let (mut window, event_loop, renderer) = window::Window::new();

    let mut main_game = match MainGame::new(renderer) {
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
            Ok(()) => {}
            Err(e) => {
                println!("{}", e);
                loop {}
            }
        }
    });
}
