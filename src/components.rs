use bevy_ecs::prelude::*;
use color_eyre::Result;
use glow::{Buffer, Context, VertexArray};
use nalgebra_glm as glm;
use tracing::warn;

use crate::gl_util;
use crate::shader::{Shader, ShaderBuilder, ShaderType};

#[derive(Component, Default, Debug, Copy, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl From<Position> for glm::Vec3 {
    fn from(value: Position) -> Self {
        glm::vec3(value.x, value.y, value.z)
    }
}

/// Rotation in degrees
#[derive(Component, Default, Debug, Copy, Clone)]
pub struct Rotation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Rotation> for glm::Vec3 {
    fn from(value: Rotation) -> Self {
        glm::vec3(value.x.to_radians(), value.y.to_radians(), value.z.to_radians())
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Scale {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Scale {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl Default for Scale {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0, z: 1.0 }
    }
}

impl From<Scale> for glm::Vec3 {
    fn from(value: Scale) -> Self {
        glm::vec3(value.x, value.y, value.z)
    }
}

#[derive(Bundle, Default)]
pub struct TransformBundle {
    pub position: Position,
    pub rotation: Rotation,
    pub scale: Scale,
}

#[derive(Component)]
pub struct Mesh {
    pub vao: VertexArray,
    pub num_indices: usize,
    pub buffers: Vec<Buffer>,
    destroyed: bool,
}

impl Mesh {
    pub fn from_tobj_mesh(gl: &Context, tobj_mesh: &tobj::Mesh) -> Self {
        let (vao, buffers) = unsafe {
            gl_util::create_vao(
                gl,
                bytemuck::cast_slice(&tobj_mesh.positions),
                &tobj_mesh.indices,
                bytemuck::cast_slice(&tobj_mesh.normals),
                bytemuck::cast_slice(&tobj_mesh.texcoords),
            )
        };
        let num_indices = tobj_mesh.indices.len();
        let buffers = buffers.to_vec();
        let destroyed = false;
        Self { vao, num_indices, buffers, destroyed }
    }

    /// # Safety
    ///
    /// The VAO and buffers of this mesh are no longer valid and should not be used.
    pub unsafe fn destroy(&mut self, gl: &Context) {
        gl_util::delete_vao(gl, self.vao, &self.buffers);
        self.destroyed = true;
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        if !self.destroyed {
            warn!("mesh was not destroyed (VAO: {:?})", self.vao);
        }
    }
}

#[derive(Component)]
pub struct StencilId(pub usize);

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct CustomShader {
    pub shader: Result<Shader>,
    pub vert_source: String,
    pub frag_source: String,
}

impl CustomShader {
    pub fn new(gl: &Context) -> Self {
        let vert_source = crate::shader::DEFAULT_VERT.to_owned();
        let frag_source = crate::shader::DEFAULT_FRAG.to_owned();
        let shader = Ok(ShaderBuilder::new(gl)
            .add_shader_source(&vert_source, ShaderType::Vertex)
            .unwrap()
            .add_shader_source(&frag_source, ShaderType::Fragment)
            .unwrap()
            .link()
            .unwrap());

        Self { shader, vert_source, frag_source }
    }
}

#[derive(Component)]
pub struct PointLight {
    pub ambient: glm::Vec3,
    pub diffuse: glm::Vec3,
    pub specular: glm::Vec3,
}

impl PointLight {
    pub fn new(ambient: glm::Vec3, diffuse: glm::Vec3, specular: glm::Vec3) -> Self {
        Self { ambient, diffuse, specular }
    }
}
