use std::fmt::Display;
use std::fs;
use std::path::Path;

use color_eyre::Result;
use color_eyre::eyre::eyre;
use glow::{Context, HasContext};
use nalgebra_glm as glm;
use tracing::warn;

pub const GEOMETRY_PASS_VERT: &str = include_str!("../shaders/geometry_pass_vert.glsl");
pub const GEOMETRY_PASS_FRAG: &str = include_str!("../shaders/geometry_pass_frag.glsl");
pub const DEFERRED_PASS_VERT: &str = include_str!("../shaders/deferred_pass_vert.glsl");
pub const DEFERRED_PASS_FRAG: &str = include_str!("../shaders/deferred_pass_frag.glsl");

pub struct Shader {
    pub program: glow::Program,
    destroyed: bool,
}

impl Shader {
    pub fn new(program: glow::Program) -> Self {
        Self { program, destroyed: false }
    }

    pub fn activate(&self, gl: &Context) {
        unsafe { gl.use_program(Some(self.program)) }
    }

    pub unsafe fn destroy(&mut self, gl: &Context) {
        unsafe {
            gl.delete_program(self.program);
            self.destroyed = true;
        }
    }

    pub unsafe fn uniform_vec3(&self, gl: &Context, name: &str, value: &glm::Vec3) {
        unsafe {
            let loc = gl.get_uniform_location(self.program, name);
            gl.uniform_3_f32_slice(loc.as_ref(), glm::value_ptr(value));
        }
    }

    pub unsafe fn uniform_mat3(&self, gl: &Context, name: &str, value: &glm::Mat3) {
        unsafe {
            let loc = gl.get_uniform_location(self.program, name);
            gl.uniform_matrix_3_f32_slice(loc.as_ref(), false, glm::value_ptr(value));
        }
    }

    pub unsafe fn uniform_mat4(&self, gl: &Context, name: &str, value: &glm::Mat4) {
        unsafe {
            let loc = gl.get_uniform_location(self.program, name);
            gl.uniform_matrix_4_f32_slice(loc.as_ref(), false, glm::value_ptr(value));
        }
    }

    pub unsafe fn uniform_float(&self, gl: &Context, name: &str, value: f32) {
        unsafe {
            let loc = gl.get_uniform_location(self.program, name);
            gl.uniform_1_f32(loc.as_ref(), value);
        }
    }

    pub unsafe fn uniform_int(&self, gl: &Context, name: &str, value: i32) {
        unsafe {
            let loc = gl.get_uniform_location(self.program, name);
            gl.uniform_1_i32(loc.as_ref(), value);
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        if !self.destroyed {
            warn!("shader program was not destroyed (Program: {:?})", self.program);
        }
    }
}

#[derive(Clone, Copy)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

impl Display for ShaderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderType::Vertex => write!(f, "Vertex"),
            ShaderType::Fragment => write!(f, "Fragment"),
        }
    }
}

pub struct ShaderBuilder<'a> {
    gl: &'a Context,
    shaders: Vec<glow::Shader>,
}

impl<'a> ShaderBuilder<'a> {
    pub fn new(gl: &'a Context) -> Self {
        Self { gl, shaders: Vec::new() }
    }

    #[allow(dead_code)]
    pub fn add_shader_file<P: AsRef<Path>>(self, path: P, shader_type: ShaderType) -> Result<Self> {
        let shader_bytes = fs::read(&path).map_err(|e| eyre!("could not add shader: {e}"))?;
        let shader_source = String::from_utf8_lossy(&shader_bytes);

        self.add_shader_source(&shader_source, shader_type)
            .map_err(|e| eyre!("{}: {e}", path.as_ref().display()))
    }

    pub fn add_shader_source(mut self, source: &str, shader_type: ShaderType) -> Result<Self> {
        let shader_enum = match shader_type {
            ShaderType::Vertex => glow::VERTEX_SHADER,
            ShaderType::Fragment => glow::FRAGMENT_SHADER,
        };

        let shader = unsafe {
            let shader = self
                .gl
                .create_shader(shader_enum)
                .map_err(|e| eyre!("could not create shader: {e}"))?;
            self.gl.shader_source(shader, source);
            self.gl.compile_shader(shader);

            if !self.gl.get_shader_compile_status(shader) {
                return Err(eyre!(
                    "{shader_type} shader compilation failed:\n{}",
                    self.gl.get_shader_info_log(shader)
                ));
            }
            shader
        };

        self.shaders.push(shader);
        Ok(self)
    }

    pub fn link(self) -> Result<Shader> {
        let program = unsafe {
            self.gl.create_program().map_err(|e| eyre!("could not create shader program: {e}"))?
        };

        for &shader in &self.shaders {
            unsafe {
                self.gl.attach_shader(program, shader);
            }
        }

        unsafe {
            self.gl.link_program(program);

            if !self.gl.get_program_link_status(program) {
                self.gl.delete_program(program);
                return Err(eyre!(
                    "shader program linking failed:\n{}",
                    self.gl.get_program_info_log(program)
                ));
            }
        }

        for shader in self.shaders {
            unsafe {
                self.gl.delete_shader(shader);
            }
        }

        Ok(Shader::new(program))
    }
}
