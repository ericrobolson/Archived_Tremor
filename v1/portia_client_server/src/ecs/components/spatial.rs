use crate::ecs::prelude::*;

use crate::math::*;

pub struct Aabb{
    min: Vec3,
    max: Vec3,
}

/// A class for object transformations. Stores the current and previous transforms to allow for interpolation.
pub struct Transformed{
    transform: Transform,
    prev_transform: Transform
}

impl Transformed{
    /// Mutable reference to the current transform.
    pub fn current_mut(&mut self) -> &mut Transform{
        &mut self.transform
    }

    /// Returns the current transform.
    pub fn current(&self) -> Transform{
        self.transform
    }

    /// Returns the previous transform.
    pub fn prev(&self) -> Transform{
        self.prev_transform
    }

    /// Copies the current transform to the previous transform.
    pub fn copy_to_previous(&mut self){
        self.prev_transform = self.transform;
    }
}

impl Component for Transformed{
    fn default(world_settings: &WorldSettings) -> Self{
        Self{
            transform: Transform::default(),
            prev_transform: Transform::default()
        }
    }
}

#[derive(Copy, Clone)]
pub struct Transform {
    pub scale: Vec3,
    pub rotation: Quaternion,
    pub position: Vec3,
}

impl Transform {
     fn default() -> Self {
        Self {
            scale: Vec3::default(),
            rotation: Quaternion::identity(),
            position: Vec3::default(),
        }
    }
}


pub struct Collidable {}

impl Component for Collidable{
    fn default(world_settings: &WorldSettings) -> Self{
        Self{}
    }
}