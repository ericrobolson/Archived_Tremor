pub mod client;
use client::{Client, Player, PlayerTypes};

pub mod lib_core;
use lib_core::LookUpGod;

pub mod event_journal;
use event_journal::EventJournal;

pub mod event_queue;
use event_queue::*;

pub mod network;
use network::socket_manager::SocketManager;

pub mod server;
use server::Server;

pub mod constants;

pub mod gfx;
use gfx::GfxRenderer;

use std::io;
use std::io::prelude::*;

fn main() {
    // Init game state
    let mut game_state = match GameState::new() {
        Ok(state) => state,
        Err(e) => {
            println!("{}", e);
            loop {}
        }
    };

    // Init gfx
    let (event_loop, window, mut gfx_state) = gfx::setup(&game_state.client.world, 60);

    // Run
    event_loop.run(move |event, _, control_flow| {
        // Parse window events
        gfx_state.handle_events(event, control_flow, &window, &mut game_state.event_queue);
        // Update game state
        match game_state.execute() {
            Ok(()) => {}
            Err(e) => {
                println!("{}", e);
                loop {}
            }
        }
        // Update gfx state.
        gfx_state.update(&game_state.client.world);
    });
}

/*
    This engine follows the model of Quake 3 (https://fabiensanglard.net/quake3/).
    A central event queue is used for communicating between systems.
*/
pub struct GameState {
    lug: LookUpGod,
    client: Client,
    server: Server,
    journal: EventJournal,
    socket_manager: SocketManager,
    pub event_queue: EventQueue,
    pub socket_out_event_queue: EventQueue,
}

impl GameState {
    pub fn new() -> Result<Self, String> {
        let socket_manager = SocketManager::new("0.0.0.0:0")?;

        let mut game = Self {
            lug: LookUpGod::new(),
            client: Client::new(),
            server: Server::new(),
            socket_manager: socket_manager,
            journal: EventJournal::new(),
            event_queue: EventQueue::new(),
            socket_out_event_queue: EventQueue::new(),
        };

        for i in 0..2 {
            game.client.add_player(Player {
                player_type: PlayerTypes::Local,
                remote_addr: None,
            })?;
        }

        Ok(game)
    }

    fn setup_from_cli(&mut self) {
        println!("Enter player type: 'client' or 'server': ");
        let mut player = Player {
            player_type: PlayerTypes::Local,
            remote_addr: None,
        };

        // TODO: actually implement client

        self.client.add_player(player).unwrap();
        return;

        let mut has_player_type = false;
        let mut is_client = false;
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line.unwrap();
            if !has_player_type {
                if line == String::from("client") {
                    is_client = true;
                    println!("Enter remote addr: ");
                    unimplemented!("IMPLEMENT CLIENT CONNECTION");

                    break;
                } else if line == String::from("server") {
                    unimplemented!("IMPLEMENT SERVER");
                    continue;
                }
            } else {
                let socket_addr: network::SocketAddr = line.parse().expect("Unable to parse addr");

                player.remote_addr = Some(socket_addr);
                break;
            }
        }
    }

    pub fn init(&mut self) -> Result<(), String> {
        self.setup_from_cli();

        Ok(())
    }

    /// Execute the game. Processes networking and state. Window input should already be written before calling this.
    pub fn execute(&mut self) -> Result<(), String> {
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

            self.server
                .execute(&mut self.event_queue, &mut self.socket_out_event_queue)?;

            self.client
                .execute(&mut self.event_queue, &mut self.socket_out_event_queue)?;
        }

        // Clear the event queue so we can start fresh next process
        self.event_queue.clear();

        Ok(())
    }
}
