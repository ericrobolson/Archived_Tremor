use crate::lib_core::{
    input::PlayerInput,
    math::FixedNumber,
    math::Vec3,
    spatial::{Camera, Transform},
    time::{GameFrame, Timer},
    voxels::{Chunk, ChunkManager, Voxel},
};

mod systems;

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
                    camera: Camera::new( (0, 10, 100).into(), (0, 0, 0).into()),
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

            pub fn max_entities(&self ) -> usize{
                MAX_ENTITIES
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
                            self.add_component(entity, Mask::PLAYER_INPUT)?;
                            self.add_component(entity, Mask::PLAYER_INPUT_ID)?;

                            self.player_input_id[entity] = input_id;

                            // Voxels
                            self.add_component(entity, Mask::VOXEL_CHUNK)?;
                            self.add_component(entity, Mask::TRANSFORM)?;
                            self.transforms[entity] = Transform::new((0,0,0).into(), Vec3::new(), Vec3::one());

                            let (x_depth, y_depth, z_depth) = self.voxel_chunks[entity].capacity();

                            let chunk = &mut self.voxel_chunks[entity];

                            for x in 0..x_depth{
                                let zs = vec![0, z_depth - 1];

                                for z in zs {
                                    chunk.set_voxel(x,0,z, Voxel::Bone);
                                    chunk.set_voxel(x, y_depth - 1,z, Voxel::Cloth);
                                }
                            }

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

                    // Execute systems
                    systems::input_register(self);
                    systems::input_actions(self);
                    systems::movement(self);
                    systems::collision_detection(self);
                    systems::collision_resolution(self);
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
        (inputs, PlayerInput, PLAYER_INPUT, 1 << 3, PlayerInput::new(), PlayerInput::new()),
        (player_input_id, usize, PLAYER_INPUT_ID, 1 << 4, 0,0),
        (transforms, Transform, TRANSFORM, 1 << 5, Transform::default(), Transform::default()),
        (velocities, Transform, VELOCITY, 1 << 6, Transform::default(), Transform::default()),

        // Voxels
        (voxel_chunks, Chunk, VOXEL_CHUNK, 1 << 16, Chunk::new(16,16,16,2), Chunk::new(16,16,16, 2)),
    ]
];
