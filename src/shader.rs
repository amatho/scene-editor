use std::ffi::CString;
use std::path::Path;
use std::{fs, ptr};

use gl::types::GLuint;

pub struct Shader {
    program_id: GLuint,
}

impl Shader {
    pub fn activate(&self) {
        unsafe {
            gl::UseProgram(self.program_id);
        }
    }
}

pub enum ShaderType {
    Vertex,
    Fragment,
}

pub struct ShaderBuilder {
    shaders: Vec<GLuint>,
}

impl ShaderBuilder {
    pub fn new() -> Self {
        Self { shaders: Vec::new() }
    }

    pub fn add_shader<P: AsRef<Path>>(
        mut self,
        path: P,
        shader_type: ShaderType,
    ) -> Result<Self, String> {
        let shader_contents = fs::read(&path).map_err(|e| format!("could not add shader: {e}"))?;
        let shader_enum = match shader_type {
            ShaderType::Vertex => gl::VERTEX_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
        };

        let shader_id = unsafe {
            let shader = gl::CreateShader(shader_enum);
            let shader_contents = CString::new(shader_contents)
                .map_err(|_| String::from("could not create CString from shader source"))?;
            gl::ShaderSource(shader, 1, &shader_contents.as_ptr(), ptr::null());
            gl::CompileShader(shader);

            let mut success = 0;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut info_log = [0u8; 512];
                gl::GetShaderInfoLog(shader, 512, ptr::null_mut(), info_log.as_mut_ptr().cast());
                return Err(format!(
                    "shader \"{}\" compilation failed:\n{}",
                    path.as_ref().display(),
                    String::from_utf8_lossy(&info_log)
                ));
            }
            shader
        };

        self.shaders.push(shader_id);
        Ok(self)
    }

    pub fn link(self) -> Result<Shader, String> {
        let program_id = unsafe { gl::CreateProgram() };

        for &shader_id in &self.shaders {
            unsafe {
                gl::AttachShader(program_id, shader_id);
            }
        }

        unsafe {
            gl::LinkProgram(program_id);

            let mut success = 0;
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut info_log = [0u8; 512];
                gl::GetProgramInfoLog(
                    program_id,
                    512,
                    ptr::null_mut(),
                    info_log.as_mut_ptr().cast(),
                );
                return Err(format!(
                    "shader program linking failed:\n{}",
                    String::from_utf8_lossy(&info_log)
                ));
            }
        }

        for shader_id in self.shaders {
            unsafe {
                gl::DeleteShader(shader_id);
            }
        }

        Ok(Shader { program_id })
    }
}
