use super::*;

pub struct Label {
    // Previously inner label
    pub text: String,
    pub font_size: f32,
    pub position: ScreenPoint,
    pub color: Color,
}

//pub type Label = ElementContainer<InnerLabel>;

impl Label {
    pub fn new(text: String, color: Color, font_size: f32, position: ScreenPoint) -> Self {
        Self {
            text,
            font_size,
            position,
            color,
        }
    }

    pub fn update_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn update_font_size(&mut self, font_size: f32) {
        self.font_size = font_size;
    }

    pub fn update_color(&mut self, color: Color) {
        self.color = color;
    }
}

impl Renderable for Label {
    fn render(&self, queue: &mut RenderQueue) {
        queue.push(RenderCommand::Text {
            font: "Tuffy_Bold.ttf",
            position: self.position,
            text: self.text.clone(),
            font_size: self.font_size,
            color: self.color,
        });
    }
}
