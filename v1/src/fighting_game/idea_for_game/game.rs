use crate::fighting_game::character_stats;
use character_stats::CharacterStats;

#[derive(Clone, Copy, Debug)]
pub struct GameRules {
    pub shield_regen: u32,
    pub num_lives: u32,
}

pub struct World {
    character_data: Vec<CharacterStats>,
    characters: Vec<Character>,
}

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    Left,
    Right,
}

pub struct Input {}

pub type RemainingFrames = u32;

#[derive(Clone, Debug)]
pub struct Character {
    rules: GameRules,

    character_data_index: usize,

    remaining_shield: u32,
    remaining_lives: u32,
    damage: u32,

    remaining_jumps: u32,
    in_air: bool,
    facing: Direction,
    position: (i32, i32),
    velocity: (i32, i32),

    state_frame: u32,
    state_id: character_stats::StateId,

    remaining_helpless: RemainingFrames,
    remaining_stun: RemainingFrames,
    remaining_grounded: RemainingFrames,
    remaining_invincibility: RemainingFrames,
    remaining_recovery: RemainingFrames,
    remaining_blockstun: RemainingFrames,
}

impl Character {
    pub fn new(game_rules: GameRules, character: CharacterStats) -> Self {
        unimplemented!()
    }

    pub fn register_input(&mut self) {
        unimplemented!();
    }

    pub fn tick(&mut self, character: &CharacterStats) {
        // Decrease remaining frames
        {
            self.remaining_helpless = checked_decrement(self.remaining_helpless);
            self.remaining_stun = checked_decrement(self.remaining_stun);
            self.remaining_grounded = checked_decrement(self.remaining_grounded);
            self.remaining_invincibility = checked_decrement(self.remaining_invincibility);
            self.remaining_recovery = checked_decrement(self.remaining_recovery);
            self.remaining_blockstun = checked_decrement(self.remaining_blockstun);
        }

        // Try to increase shield health
        {
            let shield_health = character.stats.shield_health;

            if self.state_id != character.required_states.ground.shielding
                && self.remaining_shield < shield_health
            {
                self.remaining_shield = add_with_max(
                    self.remaining_shield,
                    self.rules.shield_regen,
                    shield_health,
                );
            }
        }

        // Try to reset jumps
        {
            let max_jumps = character.stats.num_jumps;

            if self.remaining_jumps < max_jumps
                && !self.in_air
                && !self.is_helpless_state(character)
            {
                self.remaining_jumps = max_jumps;
            }
        }

        // Do collision checks

        // Apply inputs

        // state transitions
    }

    /// Returns whether the character is in a 'helpless' state and can't do anything
    fn is_helpless_state(&self, character: &CharacterStats) -> bool {
        if self.remaining_helpless > 0
            || self.remaining_stun > 0
            || self.remaining_grounded > 0
            || self.remaining_recovery > 0
            || self.remaining_blockstun > 0
        {
            return true;
        }

        false
    }
}

fn checked_decrement(value: u32) -> u32 {
    if let Some(value) = value.checked_sub(1) {
        value
    } else {
        0
    }
}

fn sub_with_min(value: u32, sub: u32, min: u32) -> u32 {
    if value == min || value < min {
        min
    } else {
        value - sub
    }
}

fn add_with_max(value: u32, addition: u32, max: u32) -> u32 {
    let mut value = value + addition;
    if value > max {
        value = max;
    }

    value
}
