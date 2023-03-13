use crate::renderer::Renderer;

pub struct State {
    window_size: (u32, u32),
}

impl State {
    pub fn new(window_size: (u32, u32)) -> Self {
        Self { window_size }
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.window_size = new_size;
    }

    pub fn render<R: Renderer>(&self, renderer: &R) {
        renderer.render(self);
    }
}
