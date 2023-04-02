use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use bevy_ecs::system::Resource;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use egui_glow::EguiGlow;
use glow::{Context, HasContext, Texture};
use nalgebra_glm as glm;
use tobj::Model;
use winit::event::{ElementState, MouseButton, VirtualKeyCode};
use winit::window::Window;

use crate::shader::{Shader, ShaderBuilder, ShaderType};

#[derive(Resource)]
pub struct RenderSettings {
    pub default_shader: Shader,
    pub outline_shader: Shader,
    pub default_texture: Texture,
}

impl RenderSettings {
    pub fn new(gl: &Context) -> Result<Self> {
        let default_shader = ShaderBuilder::new(gl)
            .add_shader_source(crate::shader::DEFAULT_VERT, ShaderType::Vertex)?
            .add_shader_source(crate::shader::DEFAULT_FRAG, ShaderType::Fragment)?
            .link()?;

        let outline_shader = ShaderBuilder::new(gl)
            .add_shader_source(include_str!("../shaders/outline_vert.glsl"), ShaderType::Vertex)?
            .add_shader_source(include_str!("../shaders/outline_frag.glsl"), ShaderType::Fragment)?
            .link()?;

        let default_texture = unsafe {
            let tex = gl.create_texture().map_err(|e| eyre!("could not create texture: {e}"))?;
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            let pixels: [u8; 4] = [229, 229, 229, 255];
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                1,
                1,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(&pixels),
            );
            tex
        };

        Ok(Self { default_shader, outline_shader, default_texture })
    }
}

#[derive(Resource, Default)]
pub struct Camera {
    pub projection: glm::Mat4,

    pub pos: glm::Vec3,
    pub front: glm::Vec3,
    pub up: glm::Vec3,

    pub yaw: f64,
    pub pitch: f64,
}

impl Camera {
    pub fn new(
        projection: glm::Mat4,
        pos: glm::Vec3,
        front: glm::Vec3,
        up: glm::Vec3,
        yaw: f64,
        pitch: f64,
    ) -> Self {
        Self { projection, pos, front, up, yaw, pitch }
    }

    pub fn perspective(width: u32, height: u32) -> glm::Mat4 {
        glm::perspective(width as f32 / height as f32, 74.0_f32.to_radians(), 0.1, 350.0)
    }
}

#[derive(Resource)]
pub struct UiState {
    pub window: Arc<Window>,
    pub egui_glow: EguiGlow,
    pub width: u32,
    pub height: u32,
    pub camera_focused: bool,
    pub side_panel_open: bool,
    pub editing_mode: Option<ShaderType>,
}

impl UiState {
    pub fn new(window: Arc<Window>, egui_glow: EguiGlow) -> Self {
        let (width, height) = window.inner_size().into();
        let camera_focused = false;
        let side_panel_open = false;
        let editing_mode = None;

        Self { window, egui_glow, width, height, camera_focused, side_panel_open, editing_mode }
    }
}

#[derive(Resource)]
pub struct ModelLoader {
    models: HashMap<String, Model>,
}

impl ModelLoader {
    pub fn new() -> Self {
        Self { models: HashMap::new() }
    }

    pub fn load_model(&mut self, name: &str) -> Result<&Model> {
        if !self.models.contains_key(name) {
            let mut path = Path::new("obj").join(name);
            path.set_extension("obj");
            let model = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS)?
                .0
                .into_iter()
                .next()
                .ok_or(eyre!("obj file had no models"))?;

            self.models.insert(name.to_owned(), model);
        }

        Ok(self.models.get(name).unwrap())
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
}
