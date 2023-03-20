use bevy_ecs::system::Resource;
use gl::types::GLuint;
use nalgebra_glm as glm;

#[derive(Resource, Default)]
pub struct Camera {
    pub view: glm::Mat4,
    pub projection: glm::Mat4,
}

impl Camera {
    pub fn new(view: glm::Mat4, projection: glm::Mat4) -> Self {
        Self { view, projection }
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
