use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use bevy_ecs::system::Resource;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use egui_glow::EguiGlow;
use glow::{Context, HasContext, Texture};
use nalgebra_glm as glm;
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
    pub fn new(gl: &Context, default_texture: Texture) -> Result<Self> {
        let default_shader = ShaderBuilder::new(gl)
            .add_shader_source(crate::shader::DEFAULT_VERT, ShaderType::Vertex)?
            .add_shader_source(crate::shader::DEFAULT_FRAG, ShaderType::Fragment)?
            .link()?;

        let outline_shader = ShaderBuilder::new(gl)
            .add_shader_source(include_str!("../shaders/outline_vert.glsl"), ShaderType::Vertex)?
            .add_shader_source(include_str!("../shaders/outline_frag.glsl"), ShaderType::Fragment)?
            .link()?;

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
    pub width: u32,
    pub height: u32,
    pub camera_focused: bool,
    pub utilities_open: bool,
    pub editing_mode: Option<ShaderType>,
    pub selected_model: Option<String>,
}

impl UiState {
    pub fn new(window: &Window) -> Self {
        let (width, height) = window.inner_size().into();
        let camera_focused = false;
        let utilities_open = false;
        let editing_mode = None;
        let selected_model = None;

        Self { width, height, camera_focused, utilities_open, editing_mode, selected_model }
    }
}

#[derive(Resource)]
pub struct EguiGlowRes {
    egui_glow: EguiGlow,
}

impl EguiGlowRes {
    pub fn new(egui_glow: EguiGlow) -> Self {
        Self { egui_glow }
    }
}

impl std::ops::DerefMut for EguiGlowRes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.egui_glow
    }
}

impl std::ops::Deref for EguiGlowRes {
    type Target = EguiGlow;

    fn deref(&self) -> &Self::Target {
        &self.egui_glow
    }
}

#[derive(Resource)]
pub struct WinitWindow {
    window: Arc<Window>,
}

impl WinitWindow {
    pub fn new(window: Arc<Window>) -> Self {
        Self { window }
    }
}

impl std::ops::DerefMut for WinitWindow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.window
    }
}

impl std::ops::Deref for WinitWindow {
    type Target = Arc<Window>;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

#[derive(Resource)]
pub struct ModelLoader {
    models: HashMap<String, tobj::Mesh>,
}

impl ModelLoader {
    pub fn new() -> Self {
        Self { models: HashMap::new() }
    }

    pub fn load_models_in_dir<P>(&mut self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        for entry in path.as_ref().read_dir()? {
            let entry = entry?;
            self.load_model(entry.path())?;
        }

        Ok(())
    }

    pub fn load_model<P>(&mut self, path: P) -> Result<()>
    where
        P: AsRef<Path> + fmt::Debug,
    {
        let (models, _) = tobj::load_obj(&path, &tobj::GPU_LOAD_OPTIONS)?;
        let model = models.into_iter().next().ok_or_else(|| eyre!("OBJ had no models"))?;
        self.models.insert(model.name, model.mesh);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&tobj::Mesh> {
        self.models.get(name)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.models.keys()
    }
}

pub struct TextureLoader {
    textures: HashMap<String, glow::Texture>,
}

impl TextureLoader {
    pub fn new() -> Self {
        Self { textures: HashMap::new() }
    }

    pub fn load_textures_in_dir<P>(&mut self, gl: &Context, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        for entry in path.as_ref().read_dir()? {
            let entry = entry?;
            self.load_texture(gl, entry.path())?;
        }

        Ok(())
    }

    pub fn load_texture<P>(&mut self, gl: &Context, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let decoder = png::Decoder::new(File::open(path.as_ref())?);
        let mut reader = decoder.read_info()?;
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf)?;

        if info.bit_depth != png::BitDepth::Eight {
            return Err(eyre!(
                "invalid bit depth {:?} of image {}",
                info.bit_depth,
                path.as_ref().display()
            ));
        }
        let source_format = match info.color_type {
            png::ColorType::Grayscale => glow::RED,
            png::ColorType::Rgb => glow::RGB,
            png::ColorType::Indexed => glow::RED,
            png::ColorType::GrayscaleAlpha => glow::RG,
            png::ColorType::Rgba => glow::RGBA,
        };

        let bytes = &buf[..info.buffer_size()];

        let texture = unsafe {
            let texture =
                gl.create_texture().map_err(|e| eyre!("could not create texture: {e}"))?;
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                source_format as i32,
                info.width as i32,
                info.height as i32,
                0,
                source_format,
                glow::UNSIGNED_BYTE,
                Some(bytes),
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR_MIPMAP_NEAREST as i32,
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            gl.generate_mipmap(glow::TEXTURE_2D);
            texture
        };

        let file_stem = path
            .as_ref()
            .file_stem()
            .ok_or_else(|| eyre!("could not get file stem"))?
            .to_string_lossy()
            .into_owned();
        self.textures.insert(file_stem, texture);

        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&Texture> {
        self.textures.get(name)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.textures.keys()
    }
}

#[derive(Resource, Default)]
pub struct Time {
    pub delta_time: f32,
}

#[derive(Resource, Default)]
pub struct Input {
    keys: HashMap<VirtualKeyCode, HeldState>,
    pub mouse_delta: (f64, f64),
    pub mouse_pos: (f64, f64),
    mouse_buttons: HashMap<MouseButton, HeldState>,
}

enum HeldState {
    Pressed,
    Held,
}

impl Input {
    pub fn handle_keyboard_input(&mut self, keycode: VirtualKeyCode, state: ElementState) {
        match state {
            ElementState::Pressed => {
                self.keys.insert(keycode, HeldState::Pressed);
            }
            ElementState::Released => {
                self.keys.remove(&keycode);
            }
        }
    }

    pub fn handle_mouse_button_input(&mut self, button: MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => {
                self.mouse_buttons.insert(button, HeldState::Pressed);
            }
            ElementState::Released => {
                self.mouse_buttons.remove(&button);
            }
        }
    }

    /// Update input state after the frame
    pub fn update_after_frame(&mut self) {
        // Keys already existing in map are now marked as held
        for val in self.keys.values_mut() {
            *val = HeldState::Held;
        }

        // Same as above
        for val in self.mouse_buttons.values_mut() {
            *val = HeldState::Held;
        }

        // Reset mouse delta to allow camera to be held still
        self.mouse_delta = (0.0, 0.0);
    }

    pub fn get_key_press(&self, keycode: VirtualKeyCode) -> bool {
        matches!(self.keys.get(&keycode), Some(HeldState::Pressed))
    }

    pub fn get_key_press_continuous(&self, keycode: VirtualKeyCode) -> bool {
        self.keys.get(&keycode).is_some()
    }

    pub fn get_mouse_button_press(&self, button: MouseButton) -> bool {
        matches!(self.mouse_buttons.get(&button), Some(HeldState::Pressed))
    }

    #[allow(dead_code)]
    pub fn get_mouse_button_press_continuous(&self, button: MouseButton) -> bool {
        self.mouse_buttons.get(&button).is_some()
    }
}
