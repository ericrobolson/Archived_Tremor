use super::*;

pub struct ForceApplication {}

impl System for ForceApplication {
    fn new(max_entities: usize) -> Self {
        Self {}
    }

    fn reset(&mut self) {}
    
}
