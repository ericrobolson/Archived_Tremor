pub mod components;

const MAX_ENTITIES: usize = 3000;

pub type Entity = usize;

pub type MaskType = u32;
struct Mask {}
impl Mask {
    pub const EMPTY: MaskType = 0;
}

macro_rules! m_world {
    (components: [$(($component_id:ident, $component_type:ty, $component_init:expr, $component_reset:expr),)*]) => {
        pub struct World{
            next_entity_id: Entity,
            initialized: bool,
            entities_to_delete: usize,
            //
            // Components
            //
            $(
                pub $component_id: Vec<$component_type>,
            )*
        }

        impl World{
            pub fn new() -> Self{
                let mut world = Self{
                    next_entity_id: 0,
                    initialized: false,
                    entities_to_delete: 0,
                    //
                    // Components
                    //
                    $(
                    $component_id: Vec::with_capacity(MAX_ENTITIES),
                    )*
                };

                world.reset();

                world
            }

            pub fn reset(&mut self){
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
            }

            pub fn dispatch(&mut self) -> Result<(), String>{
                // TODO: execute systems

                for i in 0..MAX_ENTITIES {
                    // Remove deleted entities
                    if self.entities_to_delete > 0 && self.deleted[i] == true{
                        self.entities_to_delete -= 1;

                        self.mask[i] = Mask::EMPTY;
                        self.deleted[i] = false;
                    }

                    // Figure out next entity id
                    if self.mask[i] == Mask::EMPTY && self.next_entity_id <= i{
                        self.next_entity_id = i;
                    }
                }

                Ok(())
            }

            pub fn add_entity(&mut self) -> Option<Entity> {
                if self.next_entity_id >= MAX_ENTITIES{
                    return None;
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

                self.mask[entity] |= mask;

                Ok(())
            }

            pub fn delete_component(&mut self, entity: Entity, mask: MaskType) -> Result<(), String>{
                if entity >= MAX_ENTITIES{
                    return Err("Out of bounds entity for deleting component".into());
                }

                self.mask[entity] = self.mask[entity] & !mask;

                Ok(())
            }
        }
    };
}

m_world![
    components: [
        // Sys components
        (mask, MaskType, Mask::EMPTY, Mask::EMPTY),
        (deleted, bool, false, false),
        // Engine components
        (positions, f32, 0.0, 0.0),
        (velocities, f32, 0.0, 0.0),
        (networked, bool, false, false),
        (network_id, Entity, 0,0),
    ]
];
