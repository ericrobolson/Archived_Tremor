mod cursor_interactable;
use cursor_interactable::{CursorInteractable, CursorInteractableMsg};

pub mod label;
pub use label::Label;
pub mod button;
use crate::input::Input;
use crate::math::{Point3, Quartenion, Vec3};
pub use button::Button;

/// A queue of commands for rendering. Can be anything from 3d models to audio to UI.
pub struct RenderQueue {
    render_commands: Vec<RenderCommand>,
}

impl RenderQueue {
    pub fn new(capacity: usize) -> Self {
        Self {
            render_commands: Vec::with_capacity(capacity),
        }
    }

    pub fn commands(&self) -> &Vec<RenderCommand> {
        &self.render_commands
    }

    pub fn drain(&mut self) -> Vec<RenderCommand> {
        let mut commands = Vec::with_capacity(self.render_commands.len());

        commands.append(&mut self.render_commands);

        commands
    }

    pub fn push(&mut self, command: RenderCommand) {
        self.render_commands.push(command);
    }

    pub fn clear(&mut self) {
        self.render_commands.clear();
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn white() -> Self {
        Self {
            r: u8::MAX,
            g: u8::MAX,
            b: u8::MAX,
            a: u8::MAX,
        }
    }
}

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        let max = u8::MAX as f32;
        let r = self.r as f32;
        let g = self.g as f32;
        let b = self.b as f32;
        let a = self.a as f32;
        [r / max, g / max, b / max, a / max]
    }
}

pub enum RenderCommand {
    Rectangle {
        position: ScreenPoint,
        rect: BoundingBox,
        color: Color,
    },
    Text {
        font_size: f32,
        position: ScreenPoint,
        font: &'static str,
        text: String,
        color: Color,
    },
    CameraUpdate {
        target: [f32; 3],
        eye: [f32; 3],
        orthographic: bool,
    },
    DebugRectangle {
        min: [f32; 3],
        max: [f32; 3],
        z: f32,
        color: [f32; 4],
    },
    GltfDraw {
        file: &'static str,
    },
    ModelDraw {
        file: &'static str,
        position: cgmath::Vector3<f32>,
        rotation: cgmath::Quaternion<f32>,
        scale: cgmath::Vector3<f32>,
    },
    Asset(AssetCommand),
}

#[derive(Copy, Clone, Debug)]
pub enum AssetCommand {
    LoadGltf {
        file: &'static str,
    },
    DropGltf {
        file: &'static str,
    },
    LoadObj {
        file: &'static str,
        max_instances: u32,
    },
    DropObj {
        file: &'static str,
    },
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// A point that represents a position on the screen. X goes from -1..1, left to right. Y goes from 1..-1, top to bottom.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ScreenPoint {
    pub x: f32,
    pub y: f32,
}

impl std::ops::Add for ScreenPoint {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Add for Point {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
pub trait Interactable {
    type Message;
    fn update(&mut self, input: &Input) -> Option<Self::Message>;
}

pub trait Renderable {
    fn render(&self, render_queue: &mut RenderQueue);
}

/// Normalized struct for widths. 0 - 1 normalized range.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sizing {
    pub height: f32,
    pub width: f32,
}

pub struct ElementContainer<TElement> {
    element: TElement,
    sizing: Sizing,
    absolute_position: ScreenPoint,
    absolute_bounding_box: BoundingBox,
}

impl<TElement> Element for ElementContainer<TElement> {
    fn absolute_position(&self) -> ScreenPoint {
        self.absolute_position
    }
    fn set_absolute_position(&mut self, absolute_position: ScreenPoint) {
        self.absolute_position = absolute_position;
    }
    fn sizing(&self) -> Sizing {
        self.sizing
    }
    fn set_sizing(&mut self, sizing: Sizing) {
        const MIN: f32 = 0.0;
        const MAX: f32 = 100.0;
        let mut sizing = sizing;
        if sizing.width < MIN {
            sizing.width = MIN;
        }
        if sizing.height < MIN {
            sizing.height = MIN;
        }

        if sizing.width > MAX {
            sizing.width = MAX;
        }
        if sizing.height > MAX {
            sizing.height = MAX;
        }

        self.sizing = sizing;
    }

    fn absolute_bounding_box(&self) -> BoundingBox {
        self.absolute_bounding_box
    }
    fn set_absolute_bounding_box(&mut self, absolute_bounding_box: BoundingBox) {
        self.absolute_bounding_box = absolute_bounding_box;
    }
}

pub trait Element {
    // Instead of just position, have a transform? Also how to do height, width, relative or absolute?
    fn absolute_position(&self) -> ScreenPoint;
    fn set_absolute_position(&mut self, point: ScreenPoint);
    fn sizing(&self) -> Sizing;
    fn set_sizing(&mut self, sizing: Sizing);
    fn absolute_bounding_box(&self) -> BoundingBox;
    fn set_absolute_bounding_box(&mut self, bb: BoundingBox);
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BoundingBox {
    min: Point,
    max: Point,
}

impl BoundingBox {
    pub fn new() -> Self {
        Self {
            min: Point { x: 0, y: 0 },
            max: Point { x: 10, y: 10 },
        }
    }

    pub fn contains_point(&self, global_position: Point, point: &Point) -> bool {
        let min = self.min + global_position;
        let max = self.max + global_position;

        min.x <= point.x && max.x >= point.x && min.y <= point.y && max.y >= point.y
    }
}
