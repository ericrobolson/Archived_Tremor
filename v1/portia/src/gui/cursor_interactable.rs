use super::*;

use crate::input::{Input, Mouse, MouseButton};

pub enum CursorInteractableMsg {
    Clicked(MouseButton),
    Released(MouseButton),
    HoverEnter,
    HoverExit,
}

pub struct CursorInteractable {
    global_position: Point,
    bounding_box: BoundingBox,
    pub held: bool,
    pub hovering: bool,
}

impl CursorInteractable {
    pub fn new() -> Self {
        Self {
            global_position: Point { x: 0, y: 0 },
            held: false,
            hovering: false,
            bounding_box: BoundingBox::new(),
        }
    }
}

impl Interactable for CursorInteractable {
    type Message = CursorInteractableMsg;

    fn update(&mut self, input: &Input) -> Option<Self::Message> {
        match input {
            Input::Mouse(mouse) => match mouse {
                Mouse::Clicked { x, y, button } => {
                    let point = Point { x: *x, y: *y };

                    if self
                        .bounding_box
                        .contains_point(self.global_position, &point)
                    {
                        self.held = true;
                        return Some(Self::Message::Clicked(*button));
                    }
                }
                Mouse::Released { x, y, button } => {
                    if self.held {
                        self.held = false;
                        return Some(Self::Message::Released(*button));
                    }
                }
                Mouse::Moved { x, y } => {
                    let point = Point { x: *x, y: *y };
                    let point_inside = self
                        .bounding_box
                        .contains_point(self.global_position, &point);

                    if !point_inside && self.hovering {
                        self.hovering = false;
                        return Some(Self::Message::HoverExit);
                    } else if point_inside && !self.hovering {
                        self.hovering = true;
                        return Some(Self::Message::HoverEnter);
                    }
                }
            },
            _ => {}
        }

        None
    }
}
