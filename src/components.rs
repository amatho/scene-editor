use bevy_ecs::prelude::*;
use glow::{Buffer, Context, VertexArray};
use nalgebra_glm::{Vec2, Vec3};

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

impl Rotation {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Component, Debug)]
pub struct Scale {
    pub x: f32,
    pub y: f32,
    pub z: f32,
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
pub struct Mesh {
    pub vao: VertexArray,
    pub num_indices: usize,
    buffers: [Buffer; 4],
}

impl Mesh {
    pub fn new(
        gl: &Context,
        vertices: &[Vec3],
        indices: &[u32],
        normals: &[Vec3],
        texture_coords: &[Vec2],
    ) -> Self {
        let (vao, buffers) =
            unsafe { gl_util::create_vao(gl, vertices, indices, normals, texture_coords) };
        let num_indices = indices.len();

        Self { vao, num_indices, buffers }
    }

    pub fn cube(gl: &Context) -> Self {
        let (models, _materials) = tobj::load_obj("obj/cube.obj", &tobj::GPU_LOAD_OPTIONS).unwrap();
        let cube_model = models.get(0).unwrap();

        Mesh::new(
            gl,
            bytemuck::cast_slice(&cube_model.mesh.positions),
            &cube_model.mesh.indices,
            bytemuck::cast_slice(&cube_model.mesh.normals),
            bytemuck::cast_slice(&cube_model.mesh.texcoords),
        )
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
    pub shader: Result<Shader, String>,
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
