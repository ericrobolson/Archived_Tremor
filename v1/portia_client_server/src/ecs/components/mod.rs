
pub mod controllable;
pub mod ghosting;
pub mod spatial;
pub mod mesh;

use crate::ecs::prelude::*;

pub trait Component {
    fn default(world_settings: &WorldSettings) -> Self;
}

/// A generic component store.
pub struct ComponentStore<C>
where
    C: Component,
{
    /// Shrunken array of components, with the active ones at the beginning. Not guaranteed to be in order of entity ids.
    components: Vec<(EntityId, C)>,
    /// Array linking entities to components. Guaranteed to be in order of entity ids.
    entity_id_array: Vec<Option<EntityId>>,

    components_len: usize,
}

impl<C> ComponentStore<C>
where
    C: Component,
{
    pub fn new(max_components: usize, world_settings: &WorldSettings) -> Self {
        let components = {
            let mut store = Vec::with_capacity(max_components);
            for _ in 0..max_components {
                store.push((0, C::default(world_settings)));
            }
            store
        };
        let entity_id_array = {
            let mut store = Vec::with_capacity(world_settings.max_entities);
            for _ in 0..world_settings.max_entities {
                store.push(None);
            }
            store
        };

        let components_len = 0;

        Self {
            components,
            entity_id_array,
            components_len,
        }
    }
}
