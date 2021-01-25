use crate::ecs::prelude::*;

pub type GhostId = u32;

/// An item that may be scoped and 'ghosted' or replicated on clients.
pub struct Ghostable {
    clients: Vec<ClientId>,
    id: GhostId,
}

impl Component for Ghostable {
    fn default(world_settings: &WorldSettings) -> Self {
        Self {
            id: 0,
            clients: Vec::with_capacity(world_settings.max_clients),
        }
    }
}

/// An item that is 'ghosted' from the server.
pub struct Ghost {
    id: GhostId,
}

impl Component for Ghost {
    fn default(world_settings: &WorldSettings) -> Self {
        Self { id: 0 }
    }
}
