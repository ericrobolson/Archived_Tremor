/// A selectable skin for a character
pub struct Skin {
    file: String,
    thumbnail: String,
}

pub struct CharacterStats {
    pub name: String,
    pub primary_skin: Skin,
    pub alternate_skins: Vec<Skin>,
    pub stats: Stats,

    pub required_states: RequiredStates,
    pub states: Vec<State>,
}

pub struct Stats {
    pub weight: u32,
    pub gravity_percentage: u32,
    pub walk_speed: u32,
    pub run_speed: u32,
    pub air_move_speed: u32,
    pub air_acceleration: u32,
    pub jump_velocity: u32,
    pub short_hop_velocity: u32,
    pub fast_fall_speed: u32,
    pub shield_health: u32,
    pub num_jumps: u32,
}

/// A handle for how states are referenced.
pub type StateId = u32;

/// States that every character must fulfill
pub struct RequiredStates {
    pub jump: StateId,
    pub special: SpecialStates,
    pub ground: GroundStates,
    pub air: AirStates,
}

pub struct GroundStates {
    pub tilts: DirectionalGroundStates,
    pub smashes: DirectionalGroundStates,
    pub jab: StateId,
    pub idle: StateId,
    pub crouch: StateId,
    pub shielding: StateId,
    pub stunned: StateId,
    pub dashing: StateId,
    pub dash_attack: StateId,
    pub grabbed: StateId,
}

pub struct SpecialStates {
    pub forward_special: StateId,
    pub up_special: StateId,
    pub down_special: StateId,
    pub neutral_special: StateId,
}

pub struct DirectionalGroundStates {
    pub forward: StateId,
    pub down: StateId,
    pub up: StateId,
}

pub struct AirStates {
    pub foward_air: StateId,
    pub back_air: StateId,
    pub up_air: StateId,
    pub down_air: StateId,
    pub air_dodge: StateId,
    pub neutral_air: StateId,
    pub helpless: StateId,
}

pub struct State {
    pub id: StateId,
    pub name: String,
    pub frames: Vec<Frame>,
    pub animation: String,
}

pub enum Input {
    Jump,
    ShortJump,
    Shield,
    Grab,
    Special(DirectionalInput),
    Tilt(DirectionalInput),
}

pub enum DirectionalInput {
    None,
    Up,
    Down,
    Forward,
    Back,
}

/// The frame data. Contains boxes, stun data, recovery data, etc. Also includes velocity to apply to character. Oriented facing right.
pub struct Frame {
    /// Optional parameter for how long this frame is active for.
    pub duration: Option<u32>,
    pub push_box: Option<Aabb>,
    pub hit_boxes: Vec<Aabb>,
    pub hurt_boxes: Vec<Aabb>,
    pub grab_boxes: Vec<Aabb>,
    pub hit_stun: u32,
    pub block_stun: u32,
    pub recovery: u32,
    pub velocity: (i32, i32),
}

/// A axis aligned bounding box. (0,0) is the center, so this is offset from that.
pub struct Aabb {
    pub x: i32,
    pub y: i32,
}
