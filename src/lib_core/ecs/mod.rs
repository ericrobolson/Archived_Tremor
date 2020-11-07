use crate::lib_core::{
    input::PlayerInput,
    math::FixedNumber,
    math::Vec3,
    spatial::{
        physics::Capsule, physics::CollisionShapes, Aabb, Camera, CollisionList, PhysicBodies,
        Transform,
    },
    time::{GameFrame, Timer},
    voxels::{Chunk, ChunkManager, Voxel},
};

#[macro_export]
macro_rules! mask {
    ($mask_type:expr, $($next_mask:expr),*) => {
        $mask_type $(| $next_mask)*
    }; //;
}

// Simple macro to get the matching entities in the world.
macro_rules! matching_entities {
    ($world:tt, $mask_type:expr) => {
        $world
            .masks
            .iter()
            .enumerate()
            .filter(|(i, mask)| **mask & $mask_type == $mask_type)
            .map(|(i, mask)| i)
    };
}

mod assemblages;
mod systems;

const MAX_ENTITIES: usize = 200;

pub type Entity = usize;
// TODO: write a simple 'join'

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
                    camera: Camera::new( (0, 10, 200).into(), (0, 0, 0).into()),
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

                // Create basic ground voxels
                {
                    match self.add_entity() {
                        Some(entity) => {
                            // Voxels
                            self.add_component(entity, Mask::VOXEL_CHUNK)?;
                            self.add_component(entity, Mask::TRANSFORM)?;
                            self.add_component(entity, Mask::BODY)?;
                            self.bodies[entity] = PhysicBodies::Static;


                            self.transforms[entity] = Transform::new((-100,-10,0).into(), Vec3::new(), Vec3::one());

                            self.voxel_chunks[entity] = Chunk::new(200, 1, 40, 2);

                            let (x_depth, y_depth, z_depth) = self.voxel_chunks[entity].capacity();

                            let chunk = &mut self.voxel_chunks[entity];

                            for x in 0..x_depth{
                                for z in 0..z_depth {
                                    chunk.set_voxel(x,0,z, Voxel::Metal);
                                }
                            }

                            self.add_component(entity, Mask::COLLISION_SHAPE)?;

                            let max_aabb = self.transforms[entity].scale * Vec3{x: x_depth.into(), y: y_depth.into(), z: z_depth.into()};

                            self.collision_shapes[entity] = CollisionShapes::Aabb (Aabb{
                                min: Vec3::new(),
                                max: max_aabb
                            });


                        }
                        None => {}
                    }
                }

                // Create capsule for testing
                 {
                    match self.add_entity() {
                        Some(entity) => {
                            // Voxels
                            self.add_component(entity, Mask::VOXEL_CHUNK)?;
                            self.add_component(entity, Mask::TRANSFORM)?;
                            self.add_component(entity, Mask::BODY)?;
                            self.bodies[entity] = PhysicBodies::Static;


                            self.transforms[entity] = Transform::new((-100,10,0).into(), Vec3::new(), Vec3::one());




                            self.add_component(entity, Mask::COLLISION_SHAPE)?;


                            let mut capsule = Capsule::new(10.into(), 100.into(), Transform::default());

                            // Init chunk from capsule
                            let radius: usize = capsule.radius.into();
                            println!("RADIUS: {}", radius);
                            let len: usize = capsule.length.into();
                            let len = radius * 2 + len; // Need to account for the end circles
                            self.voxel_chunks[entity] = Chunk::new(len, 4 * radius, 4 * radius, 2);
                            let (x_depth, y_depth, z_depth) = self.voxel_chunks[entity].capacity();
                            let chunk = &mut self.voxel_chunks[entity];


                            for x in 0..x_depth{
                                for y in 0..y_depth{
                                    chunk.set_voxel(x,y,0, Voxel::Metal);


                                    for z in 0..z_depth {
                                        let point = Vec3{ x: x.into(), y: y.into(), z: z.into()};
                                        let scaled_point = Vec3{ x: x_depth.into(), y: y_depth.into(), z: z_depth.into()};
                                        let mut scaled_point = scaled_point / 2.into();
                                        scaled_point.x = 0.into();
                                        let point = point - scaled_point;
                                        // Transform the point to be gucci

                                        if capsule.contains_point(point){
                                            chunk.set_voxel(x,y,z, Voxel::DebugCollisionShape);
                                        }

                                    }
                                }
                            }



                            capsule.update_transform(self.transforms[entity]);
                            self.collision_shapes[entity] = CollisionShapes::Capsule (capsule);


                        }
                        None => {}
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
                            self.add_component(entity, Mask::VELOCITY)?;

                            self.add_component(entity, Mask::TRACKABLE)?;

                            self.add_component(entity, Mask::BODY)?;
                            self.bodies[entity] = PhysicBodies::Kinematic;

                            let x_pos = entity * 25;

                            self.transforms[entity] = Transform::new((x_pos as i32,10,0).into(), Vec3::new(), Vec3::one());

                            let (x_depth, y_depth, z_depth) = self.voxel_chunks[entity].capacity();

                            let chunk = &mut self.voxel_chunks[entity];

                            for x in 0..x_depth{
                                let zs = vec![0, z_depth - 1];

                                for z in zs {
                                    chunk.set_voxel(x,0,z, Voxel::Bone);
                                    chunk.set_voxel(x, y_depth - 1,z, Voxel::Cloth);
                                }
                            }

                            for x in 0..x_depth{
                                for y in 0..y_depth{
                                    for z in 0..z_depth{
                                        if chunk.voxel(x,y,z) == Voxel::Empty{
                                            chunk.set_voxel(x, y, z, Voxel::DebugCollisionShape);
                                        }
                                    }
                                }
                            }

                            self.add_component(entity, Mask::COLLISION_SHAPE)?;

                            let max_aabb = self.transforms[entity].scale * Vec3{x: x_depth.into(), y: y_depth.into(), z: z_depth.into()};

                            self.collision_shapes[entity] = CollisionShapes::Aabb (Aabb{
                                min: Vec3::new(),
                                max: max_aabb
                            });

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
                    systems::force_application(self);
                    systems::collision_detection(self);
                    systems::collision_resolution(self);
                    systems::camera_movement(self);
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

            pub fn has_component(&self, entity: Entity, mask: MaskType) -> bool {
                if entity >= MAX_ENTITIES {
                    return false;
                }

                return self.masks[entity] & mask == mask;
            }

            pub fn remove_component(&mut self, entity: Entity, mask: MaskType) -> Result<(), String>{
                if entity >= MAX_ENTITIES{
                    return Err("Out of bounds entity for deleting component".into());
                }

                let m = self.masks[entity];

                self.masks[entity] = m & !mask;

                Ok(())
            }
        }
    };
}

//TODO: Figure out a way to get rid of the manually specified bitshifting

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
        (forces, Transform, FORCE, 1 << 7, Transform::default(), Transform::default()),
        (collision_shapes, CollisionShapes, COLLISION_SHAPE, 1 << 8, CollisionShapes::Circle{radius: 0.into()}, CollisionShapes::Circle{radius: 0.into()}),
        (collision_lists, CollisionList, COLLISIONS, 1 << 9, CollisionList::new(), CollisionList::new()),

        // Entity is trackable by the camera
        (trackable, bool, TRACKABLE, 1 << 12, true, true),
        (bodies, PhysicBodies, BODY, 1 << 13, PhysicBodies::Static, PhysicBodies::Static),


        // Voxels
        (voxel_chunks, Chunk, VOXEL_CHUNK, 1 << 16, Chunk::new(16,16,16,2), Chunk::new(16,16,16, 2)),
    ]
];
