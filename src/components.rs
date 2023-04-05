use bevy_ecs::prelude::*;
use color_eyre::Result;
use glow::{Buffer, Context, VertexArray};
use nalgebra_glm as glm;
use tobj::Model;

use crate::gl_util;
use crate::shader::{Shader, ShaderBuilder, ShaderType};

#[derive(Component, Default, Debug)]
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

#[derive(Component, Default, Debug)]
pub struct Rotation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Component, Debug)]
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

#[derive(Bundle, Default)]
pub struct TransformBundle {
    pub position: Position,
    pub rotation: Rotation,
    pub scale: Scale,
}

#[derive(Component)]
pub struct UnloadedMesh {
    vertices: Box<[glm::Vec3]>,
    indices: Box<[u32]>,
    normals: Box<[glm::Vec3]>,
    texture_coords: Box<[glm::Vec2]>,
}

impl UnloadedMesh {
    pub fn new(
        vertices: &[glm::Vec3],
        indices: &[u32],
        normals: &[glm::Vec3],
        texture_coords: &[glm::Vec2],
    ) -> Self {
        let vertices = vertices.into();
        let indices = indices.into();
        let normals = normals.into();
        let texture_coords = texture_coords.into();
        Self { vertices, indices, normals, texture_coords }
    }
}

impl From<&Model> for UnloadedMesh {
    /// Create a new `UnloadedMesh` from the given `Model`.
    fn from(model: &Model) -> Self {
        let vertices = bytemuck::cast_slice(&model.mesh.positions);
        let indices = model.mesh.indices.as_slice();
        let normals = bytemuck::cast_slice(&model.mesh.normals);
        let texture_coords = bytemuck::cast_slice(&model.mesh.texcoords);

        UnloadedMesh::new(vertices, indices, normals, texture_coords)
    }
}

#[derive(Component)]
pub struct Mesh {
    pub vao: VertexArray,
    pub num_indices: usize,
    pub buffers: Vec<Buffer>,
}

impl Mesh {
    pub fn new(gl: &Context, unloaded_mesh: &UnloadedMesh) -> Self {
        let (vao, buffers) = unsafe {
            gl_util::create_vao(
                gl,
                &unloaded_mesh.vertices,
                &unloaded_mesh.indices,
                &unloaded_mesh.normals,
                &unloaded_mesh.texture_coords,
            )
        };
        let num_indices = unloaded_mesh.indices.len();
        let buffers = buffers.to_vec();
        Self { vao, num_indices, buffers }
    }

    /// # Safety
    ///
    /// The VAO and buffers of this mesh are no longer valid and should not be used.
    pub unsafe fn destroy(&self, gl: &Context) {
        gl_util::delete_vao(gl, self.vao, &self.buffers);
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
    pub color: glm::Vec3,
}

impl PointLight {
    pub fn new(color: glm::Vec3) -> Self {
        Self { color }
    }
}
