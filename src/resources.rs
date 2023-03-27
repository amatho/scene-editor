use std::collections::HashSet;

use bevy_ecs::system::Resource;
use nalgebra_glm as glm;
use winit::event::{ElementState, MouseButton, VirtualKeyCode};

use crate::shader::Shader;

#[derive(Resource)]
pub struct ShaderState {
    pub shader: Shader,
}

impl ShaderState {
    pub fn new(shader: Shader) -> Self {
        Self { shader }
    }
}

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

#[derive(Resource)]
pub struct WindowState {
    pub width: u32,
    pub height: u32,
    pub camera_focused: bool,
}

impl WindowState {
    pub fn new(width: u32, height: u32, camera_focused: bool) -> Self {
        Self { width, height, camera_focused }
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
    pub mouse_pos: (f64, f64),
    mouse_buttons: HashSet<MouseButton>,
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

    pub fn handle_mouse_button_input(&mut self, button: MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => {
                self.mouse_buttons.insert(button);
            }
            ElementState::Released => {
                self.mouse_buttons.remove(&button);
            }
        }
    }

    pub fn handle_mouse_move(&mut self, position: (f64, f64)) {
        self.mouse_pos = position;
    }

    pub fn get_key_press(&mut self, keycode: VirtualKeyCode) -> bool {
        self.keys.remove(&keycode)
    }

    pub fn get_key_press_continuous(&self, keycode: VirtualKeyCode) -> bool {
        self.keys.contains(&keycode)
    }

    pub fn get_mouse_button_press(&mut self, button: MouseButton) -> bool {
        self.mouse_buttons.remove(&button)
    }

    pub fn get_mouse_button_press_continuous(&self, button: MouseButton) -> bool {
        self.mouse_buttons.contains(&button)
    }

    pub fn get_mouse_position(&self) -> (f64, f64) {
        self.mouse_pos
    }
}
