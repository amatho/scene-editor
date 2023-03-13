use crate::state::State;

pub trait Renderer {
    fn render(&self, state: &State);
}

pub struct MainRenderer;

impl Renderer for MainRenderer {
    fn render(&self, _state: &State) {
        // TODO: Implement
    }
}
