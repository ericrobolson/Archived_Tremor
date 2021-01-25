use super::*;
use crate::input::Input;
pub type ButtonMsg = CursorInteractableMsg;

pub struct Button {
    held: bool,
    label: Label,
    cursor_interactable: CursorInteractable,
    color: Color,
    hover_color: Color,
    held_color: Color,
    position: ScreenPoint,
    bounding_box: BoundingBox,
}

impl Button {
    pub fn new(label: Label, color: Color, hover_color: Color, held_color: Color) -> Self {
        let position = ScreenPoint { x: 0.0, y: 0.0 };
        let bounding_box = BoundingBox::new();

        let cursor_interactable = CursorInteractable::new();
        Self {
            held: false,
            label,
            color,
            hover_color,
            held_color,
            cursor_interactable,
            position,
            bounding_box,
        }
    }
}

impl Interactable for Button {
    type Message = ButtonMsg;
    fn update(&mut self, input: &Input) -> Option<Self::Message> {
        match self.cursor_interactable.update(&input) {
            Some(msg) => {
                return Some(msg);
            }
            None => {}
        }

        None
    }
}

impl Renderable for Button {
    fn render(&self, queue: &mut RenderQueue) {
        let fill_color = {
            if self.cursor_interactable.held {
                self.held_color
            } else if self.cursor_interactable.hovering {
                self.hover_color
            } else {
                self.color
            }
        };

        let fill_render = RenderCommand::Rectangle {
            position: self.position,
            rect: self.bounding_box,
            color: fill_color,
        };

        queue.push(fill_render);
        self.label.render(queue);
    }
}
