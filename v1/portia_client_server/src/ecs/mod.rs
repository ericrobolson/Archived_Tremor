use crate::ClientId;

mod components;

use components::{
    ComponentStore,
    controllable::Controllable,
    ghosting::{Ghost, Ghostable},
    spatial::{Collidable, Transformed},
    mesh::Mesh
};

pub type EntityId = usize;

pub mod prelude {
    pub use super::{ WorldSettings};
    pub use crate::ClientId;
    pub use super::EntityId;
    pub use super::components::{
        Component,
    };
}

pub struct WorldSettings {
    pub max_clients: usize,
    pub max_entities: usize,
}

pub struct World {
    ghostables: ComponentStore<Ghostable>,
    ghosts: ComponentStore<Ghost>,
    controllable: ComponentStore<Controllable>,
    transforms: ComponentStore<Transformed>,
    collidables: ComponentStore<Collidable>
}

impl World {
    pub fn new(world_settings: &WorldSettings) -> Self {
        Self {
            ghostables: ComponentStore::new(world_settings.max_entities, world_settings),
            ghosts: ComponentStore::new(world_settings.max_entities, world_settings),
            controllable: ComponentStore::new(world_settings.max_entities, world_settings),
            transforms: ComponentStore::new(world_settings.max_entities, world_settings),
            collidables: ComponentStore::new(world_settings.max_entities, world_settings)
        }
    }
}
