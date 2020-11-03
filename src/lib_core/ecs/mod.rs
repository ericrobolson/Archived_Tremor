use crate::lib_core::{
    input::PlayerInput,
    math::FixedNumber,
    math::Vec3,
    spatial::{Camera, Transformation},
    time::{GameFrame, Timer},
    voxels::{Chunk, ChunkManager, Voxel},
};

pub mod components;

const MAX_ENTITIES: usize = 200;

pub type Entity = usize;

macro_rules! m_world {
    (components: [$(($component_id:ident, $component_type:ty, $mask_name:ident, $mask_value:expr, $component_init:expr, $component_reset:expr),)*]) => {
        pub type MaskType = u32;
        pub struct Mask {}
        impl Mask {
            $(
                pub const $mask_name: MaskType = $mask_value;
            )*
        }

        pub struct World{
            next_entity_id: Entity,
            initialized: bool,
            entities_to_delete: usize,
            timer: Timer,
            frame: GameFrame,
            // Singular components
            pub world_voxels: ChunkManager,
            pub camera: Camera,
            //
            // Components
            //
            $(
                pub $component_id: Vec<$component_type>,
            )*
        }

        impl World{
            pub fn new(sim_hz: u32) -> Self{
                let mut world = Self{
                    timer: Timer::new(sim_hz),
                    next_entity_id: 0,
                    initialized: false,
                    entities_to_delete: 0,
                    frame: 0,
                    // Singular components
                    world_voxels: ChunkManager::new(16, 8, 16),
                    camera: Camera::new(),
                    //
                    // Components
                    //
                    $(
                    $component_id: Vec::with_capacity(MAX_ENTITIES),
                    )*
                };

                world.reset().unwrap();

                world
            }

            pub fn serialize(&self) -> Vec<u8> {
                unimplemented!("TODO: serialization")
            }

            pub fn deserialize(bytes: &Vec<u8>) -> Result<Self, String> {
                unimplemented!("TODO: serialization")
            }

            pub fn reset(&mut self) -> Result<(), String>{
                for i in 0..MAX_ENTITIES{
                    if !self.initialized{
                        $(
                        self.$component_id.push($component_init);
                        )*
                    } else{
                        $(
                        self.$component_id[i] = $component_reset;
                        )*
                    }
                }

                self.initialized = true;
                self.entities_to_delete = 0;
                self.frame = 0;


                Ok(())
            }

            pub fn add_player(&mut self, input_id: usize) -> Result<Option<Entity>, String>{
                {
                    match self.add_entity() {
                        Some(entity) => {
                            self.add_component(entity, Mask::POSITION)?;
                            self.add_component(entity, Mask::PLAYER_INPUT)?;
                            self.add_component(entity, Mask::PLAYER_INPUT_ID)?;

                            //self.positions[entity] = (320.0, 240.0);
                            self.player_input_id[entity] = input_id;

                            return Ok(Some(entity));
                        }
                        None => {}
                    }
                }

                Ok(None)
            }

            pub fn dispatch(&mut self) -> Result<(), String>{
                if self.timer.can_run(){
                    self.frame += 1;

                    self.world_voxels.update_frame(self.frame);

                    // simple movement system
                    {
                        const INPUT_MOVE_MASK: MaskType = Mask::POSITION & Mask::PLAYER_INPUT;
                         for entity in self.masks
                                            .iter()
                                            .enumerate()
                                            .filter(|(i, mask)| **mask & INPUT_MOVE_MASK == INPUT_MOVE_MASK)
                                            .map(|(i, mask)| i)
                        {
                            // TODO: change 'actionX' to actual input name
                            const MOVE_SPEED: f32 = 3.0;
                            /*
                            if self.inputs[entity].up {
                                let (x, y) = self.positions[entity];
                                self.positions[entity] = (x, y + MOVE_SPEED);
                            } else if  self.inputs[entity].down {
                                let (x, y) = self.positions[entity];
                                self.positions[entity] = (x, y - MOVE_SPEED);
                            }

                            if self.inputs[entity].left {
                                let (x, y) = self.positions[entity];
                                self.positions[entity] = (x - MOVE_SPEED, y);
                            } else if  self.inputs[entity].right {
                                let (x, y) = self.positions[entity];
                                self.positions[entity] = (x + MOVE_SPEED, y);
                            }
                            */
                        }
                    }


                    // Do all distance calculations after everything else has processed
                    self.world_voxels.calculate_distance_fields();
                }

                for i in 0..MAX_ENTITIES {
                    // Remove deleted entities
                    if self.entities_to_delete > 0 && self.deleted[i] == true{
                        self.entities_to_delete -= 1;

                        self.masks[i] = Mask::EMPTY;
                        self.deleted[i] = false;
                    }

                    // Figure out next entity id
                    if self.masks[i] == Mask::EMPTY && self.next_entity_id <= i{
                        self.next_entity_id = i;
                    }
                }

                Ok(())
            }

            pub fn add_entity(&mut self) -> Option<Entity> {
                if self.next_entity_id >= MAX_ENTITIES{
                    return None;
                }

                for entity in self.masks.iter().enumerate().filter(|(_i, mask)| **mask == Mask::EMPTY).map(|(entity, _)| entity){
                    return Some(entity);
                }

                None
            }

            pub fn delete_entity(&mut self, entity: Entity) -> Result<(), String>{
                if entity >= MAX_ENTITIES{
                    return Err("Attempted to delete out of bounds entity.".into());
                }

                self.deleted[entity] = true;
                self.entities_to_delete += 1;

                Ok(())
            }

            pub fn add_component(&mut self, entity: Entity, mask: MaskType) -> Result<(), String>{
                if entity >= MAX_ENTITIES{
                    return Err("Out of bounds entity for adding component".into());
                }

                self.masks[entity] |= mask;

                Ok(())
            }

            pub fn delete_component(&mut self, entity: Entity, mask: MaskType) -> Result<(), String>{
                if entity >= MAX_ENTITIES{
                    return Err("Out of bounds entity for deleting component".into());
                }

                self.masks[entity] = self.masks[entity] & !mask;

                Ok(())
            }
        }
    };
}

//TODO: Figure out a way to get rid of the manually specified bitshifting

use crate::lib_core::shapes::Csg;
m_world![
    components: [
        // Sys components
        (masks, MaskType, EMPTY, 0 << 0, Mask::EMPTY, Mask::EMPTY),
        (deleted, bool, DELETED, 1 << 0, false, false),
        // Engine components
        (positions, Vec3, POSITION, 1 << 1, Vec3::new(), Vec3::new()), // TODO: remove
        (velocities,  (f32, f32), VELOCITY, 1 << 2,(0.0, 0.0), (0.0, 0.0)),
        (inputs, PlayerInput, PLAYER_INPUT, 1 << 3, PlayerInput::new(), PlayerInput::new()),
        (player_input_id, usize, PLAYER_INPUT_ID, 1 << 4, 0,0),
        (transformations, Transformation, TRANSFORMATION, 1 << 5, Transformation::default(), Transformation::default()),
        (transformation_velocities, Transformation, TRANSFORMATION_VEL, 1 << 6, Transformation::default(), Transformation::default()),

        // Voxels
        (voxel_chunks, Chunk, VOXEL_CHUNK, 1 << 16, Chunk::new(16,16,16,2), Chunk::new(16,16,16, 2)),
    ]
];
