use std::fs;
use std::path::Path;

use glow::{Context, HasContext};

pub struct Shader {
    pub program: glow::Program,
}

impl Shader {
    pub fn activate(&self, gl: &Context) {
        unsafe { gl.use_program(Some(self.program)) }
    }
}

pub enum ShaderType {
    Vertex,
    Fragment,
}

pub struct ShaderBuilder<'a> {
    gl: &'a Context,
    shaders: Vec<glow::Shader>,
}

impl<'a> ShaderBuilder<'a> {
    pub fn new(gl: &'a Context) -> Self {
        Self { gl, shaders: Vec::new() }
    }

    pub fn add_shader<P: AsRef<Path>>(
        mut self,
        path: P,
        shader_type: ShaderType,
    ) -> Result<Self, String> {
        let shader_bytes = fs::read(&path).map_err(|e| format!("could not add shader: {e}"))?;
        let shader_source = String::from_utf8_lossy(&shader_bytes);
        let shader_enum = match shader_type {
            ShaderType::Vertex => glow::VERTEX_SHADER,
            ShaderType::Fragment => glow::FRAGMENT_SHADER,
        };

        let shader = unsafe {
            let shader = self.gl.create_shader(shader_enum)?;
            self.gl.shader_source(shader, &shader_source);
            self.gl.compile_shader(shader);

            if !self.gl.get_shader_compile_status(shader) {
                return Err(format!(
                    "shader \"{}\" compilation failed:\n{}",
                    path.as_ref().display(),
                    self.gl.get_shader_info_log(shader)
                ));
            }
            shader
        };

        self.shaders.push(shader);
        Ok(self)
    }

    pub fn link(self) -> Result<Shader, String> {
        let program = unsafe { self.gl.create_program()? };

        for &shader in &self.shaders {
            unsafe {
                self.gl.attach_shader(program, shader);
            }
        }

        unsafe {
            self.gl.link_program(program);

            if !self.gl.get_program_link_status(program) {
                return Err(format!(
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

        Ok(Shader { program })
    }
}
