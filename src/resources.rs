use std::fmt;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ahash::AHashMap;
use bevy_ecs::system::Resource;
use bevy_ecs::world::{FromWorld, World};
use color_eyre::Result;
use color_eyre::eyre::eyre;
use egui_glow::EguiGlow;
use glow::{Context, Framebuffer, HasContext, Renderbuffer, Texture};
use nalgebra_glm as glm;
use winit::event::{ElementState, MouseButton, VirtualKeyCode};
use winit::window::Window;
use zune_png::PngDecoder;
use zune_png::zune_core::bit_depth::{BitDepth, ByteEndian};
use zune_png::zune_core::colorspace::ColorSpace;
use zune_png::zune_core::options::DecoderOptions;

use crate::shader::{Shader, ShaderBuilder, ShaderType};
use crate::vao::VertexArrayObject;

#[derive(Resource)]
pub struct RenderState {
    pub default_diffuse: Texture,
    pub default_specular: Texture,
    pub shadow_map_fbo: Framebuffer,
    pub shadow_map: Texture,
    pub shadow_map_size: (i32, i32),
    pub depth_shader: Shader,
    pub g_buffer: Framebuffer,
    pub g_position: Texture,
    pub g_normal: Texture,
    pub g_albedo_spec: Texture,
    pub g_rbo: Renderbuffer,
    pub geometry_pass_shader: Shader,
    pub quad_vao: VertexArrayObject,
    pub deferred_pass_shader: Shader,
}

impl RenderState {
    pub fn new(gl: &Context, window_size: (u32, u32)) -> Result<Self> {
        let default_diffuse = unsafe {
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

        let default_specular = unsafe {
            let tex = gl.create_texture().map_err(|e| eyre!("could not create texture: {e}"))?;
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            let pixels: [u8; 4] = [0; 4];
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

        let shadow_map_size = (4096, 4096);
        let (shadow_map_fbo, shadow_map) = unsafe {
            let fbo =
                gl.create_framebuffer().map_err(|e| eyre!("could not create framebuffer: {e}"))?;

            let map = gl.create_texture().map_err(|e| eyre!("could not create texture: {e}"))?;
            gl.bind_texture(glow::TEXTURE_2D, Some(map));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::DEPTH_COMPONENT24 as i32,
                shadow_map_size.0,
                shadow_map_size.1,
                0,
                glow::DEPTH_COMPONENT,
                glow::FLOAT,
                None,
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_COMPARE_MODE,
                glow::COMPARE_REF_TO_TEXTURE as i32,
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_COMPARE_FUNC, glow::LEQUAL as i32);
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_BORDER as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_BORDER as i32,
            );
            gl.tex_parameter_f32_slice(
                glow::TEXTURE_2D,
                glow::TEXTURE_BORDER_COLOR,
                &[1.0, 1.0, 1.0, 1.0],
            );

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::DEPTH_ATTACHMENT,
                glow::TEXTURE_2D,
                Some(map),
                0,
            );
            gl.draw_buffer(glow::NONE);
            gl.read_buffer(glow::NONE);
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            (fbo, map)
        };

        let depth_shader = ShaderBuilder::new(gl)
            .add_shader_source(include_str!("../shaders/depth_vert.glsl"), ShaderType::Vertex)?
            .add_shader_source(include_str!("../shaders/depth_frag.glsl"), ShaderType::Fragment)?
            .link()?;

        let (g_buffer, g_position, g_normal, g_albedo_spec, g_rbo) = unsafe {
            let g_buf =
                gl.create_framebuffer().map_err(|e| eyre!("could not create framebuffer: {e}"))?;
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(g_buf));

            let (width, height) = window_size;
            let width = width as i32;
            let height = height as i32;
            let g_pos = gl.create_texture().map_err(|e| eyre!("could not create texture: {e}"))?;
            gl.bind_texture(glow::TEXTURE_2D, Some(g_pos));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA16F as i32,
                width,
                height,
                0,
                glow::RGBA,
                glow::FLOAT,
                None,
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(g_pos),
                0,
            );

            let g_norm = gl.create_texture().map_err(|e| eyre!("could not create texture: {e}"))?;
            gl.bind_texture(glow::TEXTURE_2D, Some(g_norm));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA16F as i32,
                width,
                height,
                0,
                glow::RGBA,
                glow::FLOAT,
                None,
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT1,
                glow::TEXTURE_2D,
                Some(g_norm),
                0,
            );

            let g_alb_spec =
                gl.create_texture().map_err(|e| eyre!("could not create texture: {e}"))?;
            gl.bind_texture(glow::TEXTURE_2D, Some(g_alb_spec));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width,
                height,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                None,
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT2,
                glow::TEXTURE_2D,
                Some(g_alb_spec),
                0,
            );

            gl.draw_buffers(&[
                glow::COLOR_ATTACHMENT0,
                glow::COLOR_ATTACHMENT1,
                glow::COLOR_ATTACHMENT2,
            ]);

            let rbo = gl
                .create_renderbuffer()
                .map_err(|e| eyre!("could not create renderbuffer: {e}"))?;
            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(rbo));
            gl.renderbuffer_storage(glow::RENDERBUFFER, glow::DEPTH24_STENCIL8, width, height);
            gl.framebuffer_renderbuffer(
                glow::FRAMEBUFFER,
                glow::DEPTH_STENCIL_ATTACHMENT,
                glow::RENDERBUFFER,
                Some(rbo),
            );

            if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
                return Err(eyre!("framebuffer was not completed"));
            }

            (g_buf, g_pos, g_norm, g_alb_spec, rbo)
        };

        let geometry_pass_shader = ShaderBuilder::new(gl)
            .add_shader_source(crate::shader::GEOMETRY_PASS_VERT, ShaderType::Vertex)?
            .add_shader_source(crate::shader::GEOMETRY_PASS_FRAG, ShaderType::Fragment)?
            .link()?;

        let quad_vertices = [
            glm::vec3(-1.0, 1.0, 0.0),
            glm::vec3(-1.0, -1.0, 0.0),
            glm::vec3(1.0, 1.0, 0.0),
            glm::vec3(1.0, -1.0, 0.0),
        ];
        let quad_indices = [0, 1, 2, 1, 3, 2];
        let quad_normals = [
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(0.0, 0.0, 0.0),
        ];
        let quad_texcoords =
            [glm::vec2(0.0, 1.0), glm::vec2(0.0, 0.0), glm::vec2(1.0, 1.0), glm::vec2(1.0, 0.0)];
        let quad_vao = unsafe {
            VertexArrayObject::new(
                gl,
                &quad_vertices,
                &quad_indices,
                &quad_normals,
                &quad_texcoords,
            )
        };

        let deferred_pass_shader = ShaderBuilder::new(gl)
            .add_shader_source(crate::shader::DEFERRED_PASS_VERT, ShaderType::Vertex)?
            .add_shader_source(crate::shader::DEFERRED_PASS_FRAG, ShaderType::Fragment)?
            .link()?;

        Ok(Self {
            default_diffuse,
            default_specular,
            shadow_map_fbo,
            shadow_map,
            shadow_map_size,
            depth_shader,
            g_buffer,
            g_position,
            g_normal,
            g_albedo_spec,
            g_rbo,
            geometry_pass_shader,
            quad_vao,
            deferred_pass_shader,
        })
    }

    pub fn resize(&mut self, gl: &Context, new_width: u32, new_height: u32) {
        let new_width = new_width as i32;
        let new_height = new_height as i32;
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.g_position));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA16F as i32,
                new_width,
                new_height,
                0,
                glow::RGBA,
                glow::FLOAT,
                None,
            );

            gl.bind_texture(glow::TEXTURE_2D, Some(self.g_normal));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA16F as i32,
                new_width,
                new_height,
                0,
                glow::RGBA,
                glow::FLOAT,
                None,
            );

            gl.bind_texture(glow::TEXTURE_2D, Some(self.g_albedo_spec));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                new_width,
                new_height,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                None,
            );

            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(self.g_rbo));
            gl.renderbuffer_storage(
                glow::RENDERBUFFER,
                glow::DEPTH24_STENCIL8,
                new_width,
                new_height,
            );
        }
    }
}

impl FromWorld for RenderState {
    fn from_world(world: &mut World) -> Self {
        let gl = world.non_send_resource::<Arc<Context>>();
        let window_size = world.resource::<WinitWindow>().inner_size().into();
        Self::new(gl, window_size).unwrap()
    }
}

#[derive(Resource)]
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

impl FromWorld for Camera {
    fn from_world(world: &mut World) -> Self {
        let size = world.resource::<WinitWindow>().inner_size();
        let projection = Self::perspective(size.width, size.height);
        Self::new(
            projection,
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(0.0, 0.0, -1.0),
            glm::vec3(0.0, 1.0, 0.0),
            -90.0,
            0.0,
        )
    }
}

#[derive(Resource, Default)]
pub struct UiState {
    pub camera_focused: bool,
    pub utilities_open: bool,
    pub performance_open: bool,
    pub editing_mode: Option<ShaderType>,
    pub selected_model: Option<String>,
    pub selected_diffuse: Option<String>,
    pub selected_specular: Option<String>,
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
    models: AHashMap<String, VertexArrayObject>,
}

impl ModelLoader {
    pub fn new() -> Self {
        Self { models: AHashMap::new() }
    }

    pub fn load_models_in_dir<P>(&mut self, gl: &Context, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        for entry in path.as_ref().read_dir()? {
            let entry = entry?;
            self.load_model(gl, entry.path())?;
        }

        Ok(())
    }

    pub fn load_model<P>(&mut self, gl: &Context, path: P) -> Result<()>
    where
        P: AsRef<Path> + fmt::Debug,
    {
        let (models, _) = tobj::load_obj(&path, &tobj::GPU_LOAD_OPTIONS)?;
        let models = models.into_iter().fuse();

        if models.len() == 0 {
            return Err(eyre!("OBJ had no models: {}", path.as_ref().display()));
        }

        for model in models {
            let vertices = bytemuck::cast_slice(&model.mesh.positions);
            let indices = &model.mesh.indices;
            let normals = bytemuck::cast_slice(&model.mesh.normals);
            let texture_coords = bytemuck::cast_slice(&model.mesh.texcoords);
            let vao =
                unsafe { VertexArrayObject::new(gl, vertices, indices, normals, texture_coords) };

            self.models.insert(model.name, vao);
        }

        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&VertexArrayObject> {
        self.models.get(name)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.models.keys()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut VertexArrayObject> {
        self.models.values_mut()
    }
}

#[derive(Resource)]
pub struct TextureLoader {
    textures: AHashMap<String, glow::Texture>,
}

impl TextureLoader {
    pub fn new() -> Self {
        Self { textures: AHashMap::new() }
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
        let contents = std::fs::read(path.as_ref())?;
        let byte_endian =
            if cfg!(target_endian = "little") { ByteEndian::LE } else { ByteEndian::BE };
        let mut decoder = PngDecoder::new_with_options(
            &contents,
            DecoderOptions::new_fast().set_byte_endian(byte_endian),
        );
        decoder.decode_headers().map_err(|_| eyre!("could not decode PNG headers"))?;

        let color_space = decoder.get_colorspace().unwrap();
        let bit_depth = decoder.get_depth().unwrap();
        let (source_format, source_type) = match (color_space, bit_depth) {
            (ColorSpace::RGB, BitDepth::Eight) => (glow::RGB, glow::UNSIGNED_BYTE),
            (ColorSpace::RGB, BitDepth::Sixteen) => (glow::RGB, glow::UNSIGNED_SHORT),
            (ColorSpace::RGBA, BitDepth::Eight) => (glow::RGBA, glow::UNSIGNED_BYTE),
            (ColorSpace::RGBA, BitDepth::Sixteen) => (glow::RGBA, glow::UNSIGNED_SHORT),
            _ => {
                return Err(eyre!(
                    "invalid bit depth {:?} of image {}",
                    bit_depth,
                    path.as_ref().display()
                ));
            }
        };

        let (width, height) = decoder.get_dimensions().unwrap();
        let bytes = decoder.decode_raw().map_err(|_| eyre!("could not decode PNG image"))?;

        let texture = unsafe {
            let texture =
                gl.create_texture().map_err(|e| eyre!("could not create texture: {e}"))?;
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width as i32,
                height as i32,
                0,
                source_format,
                source_type,
                Some(&bytes),
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

#[derive(Resource)]
pub struct Time {
    prev_frame_time: Instant,
    prev_avg_frame_time: Instant,
    frame_count: u32,
    avg_frame_time_ms: f32,
    delta_time: Duration,
}

impl Time {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            prev_frame_time: now,
            prev_avg_frame_time: now,
            frame_count: 0,
            avg_frame_time_ms: 0.0,
            delta_time: Duration::ZERO,
        }
    }

    pub fn next_frame(&mut self) {
        let now = Instant::now();
        self.delta_time = now.duration_since(self.prev_frame_time);
        self.prev_frame_time = now;

        self.frame_count += 1;
        if now.duration_since(self.prev_avg_frame_time) >= Duration::from_secs(1) {
            self.avg_frame_time_ms = 1000.0 / self.frame_count as f32;
            self.frame_count = 0;
            self.prev_avg_frame_time = now;
        }
    }

    pub fn delta_seconds(&self) -> f32 {
        self.delta_time.as_secs_f32()
    }

    pub fn avg_frame_time_ms(&self) -> f32 {
        self.avg_frame_time_ms
    }
}

impl FromWorld for Time {
    fn from_world(_world: &mut World) -> Self {
        Self::new()
    }
}

#[derive(Resource, Default)]
pub struct Input {
    keys: AHashMap<VirtualKeyCode, HeldState>,
    pub mouse_delta: (f64, f64),
    pub mouse_pos: (f64, f64),
    mouse_buttons: AHashMap<MouseButton, HeldState>,
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
