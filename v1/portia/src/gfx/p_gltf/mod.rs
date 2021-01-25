mod animations;
mod boundingbox;
mod manager;
mod material;
mod mesh;
mod node;
mod primitive;
mod scene;
mod skin;
mod textures;
mod vert_indices;

pub use vert_indices::Vertex;

pub use manager::GltfManager;

const MAX_NUM_JOINTS: usize = 128;

use std::sync::{Arc, RwLock};
type SmartPointer<T> = Arc<RwLock<T>>;
