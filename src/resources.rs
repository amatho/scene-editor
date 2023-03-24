use std::collections::HashSet;

use bevy_ecs::system::Resource;
use gl::types::GLuint;
use nalgebra_glm as glm;
use winit::event::{ElementState, VirtualKeyCode};

#[derive(Resource, Default)]
pub struct Camera {
    pub projection: glm::Mat4,

    pub pos: glm::Vec3,
    pub front: glm::Vec3,
    pub up: glm::Vec3,

    pub yaw: f32,
    pub pitch: f32,
}

impl Camera {
    pub fn new(
        projection: glm::Mat4,
        pos: glm::Vec3,
        front: glm::Vec3,
        up: glm::Vec3,
        yaw: f32,
        pitch: f32,
    ) -> Self {
        Self { projection, pos, front, up, yaw, pitch }
    }
}

#[derive(Resource, Default)]
pub struct ShaderState {
    pub program_id: GLuint,
}

impl ShaderState {
    pub fn new(program_id: GLuint) -> Self {
        Self { program_id }
    }
}

#[derive(Resource, Default)]
pub struct Time {
    pub delta_time: f32,
}

#[derive(Resource, Default)]
pub struct Input {
    keys: HashSet<VirtualKeyCode>,
    pub mouse_delta: (f64, f64),
}

impl Input {
    pub fn handle_keyboard_input(&mut self, keycode: VirtualKeyCode, state: ElementState) {
        match state {
            ElementState::Pressed => {
                self.keys.insert(keycode);
            }
            ElementState::Released => {
                self.keys.remove(&keycode);
            }
        }
    }

    pub fn is_pressed(&self, keycode: VirtualKeyCode) -> bool {
        self.keys.contains(&keycode)
    }
}
