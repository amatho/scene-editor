use bevy_ecs::prelude::*;
use gl::types::GLuint;
use nalgebra_glm::{Vec2, Vec3};

use crate::gl_util;

#[derive(Component, Default)]
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

#[derive(Component, Default)]
pub struct Rotation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Component)]
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

#[derive(Component, Default)]
pub struct Mesh {
    pub vao_id: GLuint,
    pub num_indices: usize,
}

impl Mesh {
    pub fn new(
        vertices: &[Vec3],
        indices: &[gl::types::GLuint],
        normals: &[Vec3],
        texture_coords: &[Vec2],
    ) -> Self {
        let vao_id = unsafe { gl_util::create_vao(vertices, indices, normals, texture_coords) };
        let num_indices = indices.len();

        Self { vao_id, num_indices }
    }
}
