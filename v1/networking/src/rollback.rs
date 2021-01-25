pub type PlayerId = u8;

pub mod prelude {
    pub use super::{
        FrameId, GameInput, PlayerId, RollbackEvent, RollbackGameState, RollbackNetcode,
        RollbackNetcodeModes,
    };
}

pub enum RollbackEvent {
    Disconnected,
}

pub trait GameInput: Copy + Clone + Default {}

pub enum RollbackNetcodeModes {
    /// Initiate a rollback every frame, comparing the saved state to the current one. If they aren't the same will panic.
    SyncTest,
    /// Lag test. Will pick a random value between the min and max latency and apply it.
    LagTest {
        min_latency: FrameId,
        max_latency: FrameId,
    },
    /// Loss test. Will randomly drop packets.
    PacketDrop {
        min_drop_rate: FrameId,
        max_drop_rate: FrameId,
    },
}

pub trait RollbackGameState<Input>: Clone + PartialEq
where
    Input: GameInput,
{
    fn new() -> Self;

    /// Register input for a player for the next frame.
    fn add_input(&mut self, player_id: PlayerId, input: Input);
    /// Tick the simulation.
    fn tick(&mut self);
}

pub type FrameId = u32;

pub struct RollbackNetcode<Game, Input>
where
    Game: RollbackGameState<Input>,
    Input: GameInput,
{
    /// Used for packet serialization checks. If the versions don't match, will simply ignore them.
    game_version: &'static str,
    rollback_modes: Vec<RollbackNetcodeModes>,
    input_delay: FrameId,
    num_remote_players: usize,
    player_inputs: Vec<InputStore<Input>>,
    state: Game,
    confirmed_state: Game,
    current_frame: FrameId,
    confirmed_frame: FrameId,
}

impl<Game, Input> RollbackNetcode<Game, Input>
where
    Game: RollbackGameState<Input>,
    Input: GameInput,
{
    pub fn new(
        game_version: &'static str,
        input_delay: FrameId,
        rollback_modes: Vec<RollbackNetcodeModes>,
    ) -> Self {
        Self {
            game_version,
            rollback_modes,
            input_delay,
            player_inputs: vec![],
            state: Game::new(),
            confirmed_frame: 0,
            confirmed_state: Game::new(),
            current_frame: 0,
            num_remote_players: 0,
        }
    }

    pub fn add_player(&mut self) -> PlayerId {
        let player_id = self.player_inputs.len() as PlayerId;

        self.player_inputs
            .push(InputStore::new(self.input_delay, player_id));

        player_id
    }

    /// Register local input for the player.
    pub fn register_local_input(&mut self, player_id: PlayerId, input: Input) {
        let target_frame = self.current_frame + self.input_delay;
        let player_index = player_id as usize;
        self.player_inputs[player_index].register_input(target_frame, InputType::Confirmed, input);

        self.queue_outgoing_input(player_id, input);
    }

    fn queue_outgoing_input(&mut self, player_id: PlayerId, input: Input) {
        //println!("Broadcast to other remote players");
    }

    fn sync_remote_inputs(&mut self) {
        //println!("TODO: get other player inputs from network");
    }

    /// Returns a reference to the current frame state.
    pub fn state(&self) -> &Game {
        &self.state
    }

    /// Tick the game.
    pub fn tick(&mut self) -> Vec<RollbackEvent> {
        for rollback_mode in &self.rollback_modes {
            match rollback_mode {
                RollbackNetcodeModes::SyncTest => {
                    unimplemented!("TODO: need to figure out sync test.");
                }
                RollbackNetcodeModes::LagTest {
                    min_latency,
                    max_latency,
                } => {
                    unimplemented!("TODO: lag test.");
                }
                RollbackNetcodeModes::PacketDrop {
                    min_drop_rate,
                    max_drop_rate,
                } => {
                    unimplemented!("TODO: need to figure drop test.");
                }
            }
        }

        self.sync_remote_inputs();

        // Check if there's a new confirmed state
        let confirmed_input_frame = self.confirmed_input_frames();

        // Only do a rollback if there's more than one frame to rollback.
        // TODO: only do rollbacks if there are remote players?
        let execute_rollback = {
            self.confirmed_frame < confirmed_input_frame
                && (confirmed_input_frame - self.confirmed_frame) != 1
        };

        if execute_rollback {
            // If so, load last confirmed state
            self.state = self.confirmed_state.clone();

            // Run it until the last confirmed input frames meet the new confirmed frame. Registering the inputs for each frame then ticking the world.
            let num_rollbacks = self.confirmed_frame..confirmed_input_frame;
            for rollback_frame in num_rollbacks.clone() {
                self.register_input_for_frame(rollback_frame);
                self.state.tick();
            }

            // Update state to the last confirmed frame
            self.confirmed_state = self.state.clone();
            self.confirmed_frame = confirmed_input_frame;

            // Catch up until current frame.
            let num_catchups = confirmed_input_frame..self.current_frame;
            for catchup_frame in num_catchups.clone() {
                self.register_input_for_frame(catchup_frame);
                self.state.tick();
            }
        } else {
            // No rollback, so just continue processing at one execution per tick.
            self.register_input_for_frame(self.current_frame);
            self.state.tick();
        }

        // Increment frame.
        // TODO: how to handle wrapping frames?
        self.current_frame += 1;

        vec![]
    }

    /// Register the input for the given frame to the state.
    fn register_input_for_frame(&mut self, frame: FrameId) {
        for input_store in self.player_inputs.iter_mut() {
            let input = input_store.get_input(frame);
            self.state.add_input(input_store.player_id, input);
        }
    }

    /// Loop through all player inputs, finding the min of all confirmed input frames.
    fn confirmed_input_frames(&self) -> FrameId {
        let mut earliest_frame = self.current_frame;

        for player_input in &self.player_inputs {
            earliest_frame = earliest_frame.min(player_input.last_confirmed);
        }

        earliest_frame
    }
}
#[derive(PartialEq)]
enum InputType {
    Confirmed,
    Predicted,
}

struct InputStore<Input>
where
    Input: GameInput,
{
    player_id: PlayerId,
    last_confirmed: FrameId,
    inputs: Vec<(InputType, Input)>,
}

impl<Input> InputStore<Input>
where
    Input: GameInput,
{
    /// Init new input store, populating the initial empty inputs from the input delay as confirmed.
    pub fn new(input_delay: FrameId, player_id: PlayerId) -> Self {
        let mut store = Self {
            player_id,
            last_confirmed: 0,
            inputs: vec![],
        };

        for frame in 0..input_delay {
            store.register_input(frame, InputType::Confirmed, Input::default());
        }

        store
    }

    /// Register input.
    pub fn register_input(&mut self, frame: FrameId, input_type: InputType, input: Input) {
        // Check to see if this input is past the last registered frame. If so, make some predictions and push it.
        let frame_idx = frame as usize;
        if frame_idx >= self.inputs.len() {
            self.predict_until_frame(frame);
            self.inputs.push((input_type, input));
        } else {
            // Update existing input
            self.inputs[frame_idx] = (input_type, input);
        }

        // Update the last_confirmed frame if possible and recalculate any predictions.
        for frame_idx in self.last_confirmed..frame {
            let frame_idx_usize = frame_idx as usize;

            match self.inputs[frame_idx_usize].0 {
                InputType::Confirmed => {
                    self.last_confirmed = frame_idx;
                }
                InputType::Predicted => {
                    self.inputs[frame_idx_usize] = (InputType::Predicted, self.predict_input());
                }
            }
        }
    }

    /// Ensure predictions are made up until the given frame.
    fn predict_until_frame(&mut self, frame: FrameId) {
        // Add any predictions that may have occurred if this input is ahead
        let frame_idx = frame as usize;

        while self.inputs.len() < frame_idx {
            println!("FRame idx: {:?}, len: {:?}", frame_idx, self.inputs.len());
            self.inputs
                .push((InputType::Predicted, self.predict_input()));
        }
    }

    /// Predict the input based on the last confirmed.
    fn predict_input(&self) -> Input {
        match self
            .inputs
            .iter()
            .rev()
            .filter(|(input_type, _)| *input_type == InputType::Confirmed)
            .next()
        {
            Some((input_type, input)) => *input,
            None => Input::default(),
        }
    }

    // Get the input for a given frame. If it doesn't exist, predict it.
    pub fn get_input(&mut self, frame: FrameId) -> Input {
        let frame_idx = frame as usize;
        if frame_idx < self.inputs.len() {
            // Get input
            return self.inputs[frame_idx].1;
        } else {
            self.inputs
                .push((InputType::Predicted, self.predict_input()));
            return self.get_input(frame);
        }
    }
}
