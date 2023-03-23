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

impl Rotation {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
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

    pub fn cube(width: f32, height: f32, depth: f32) -> Self {
        let half_width = width / 2.0;
        let half_height = height / 2.0;
        let half_depth = depth / 2.0;

        let vertices = [
            // Front face
            Vec3::new(-half_width, -half_height, half_depth), // Bottom left front
            Vec3::new(half_width, -half_height, half_depth),  // Bottom right front
            Vec3::new(-half_width, half_height, half_depth),  // Top left front
            Vec3::new(-half_width, half_height, half_depth),  // Top left front
            Vec3::new(half_width, -half_height, half_depth),  // Bottom right front
            Vec3::new(half_width, half_height, half_depth),   // Top right front
            // Left face
            Vec3::new(-half_width, -half_height, -half_depth), // Bottom left back
            Vec3::new(-half_width, -half_height, half_depth),  // Bottom left front
            Vec3::new(-half_width, half_height, -half_depth),  // Top left back
            Vec3::new(-half_width, half_height, -half_depth),  // Top left back
            Vec3::new(-half_width, -half_height, half_depth),  // Bottom left front
            Vec3::new(-half_width, half_height, half_depth),   // Top left front
            // Right face
            Vec3::new(half_width, -half_height, half_depth), // Bottom right front
            Vec3::new(half_width, -half_height, -half_depth), // Bottom right back
            Vec3::new(half_width, half_height, half_depth),  // Top right front
            Vec3::new(half_width, half_height, half_depth),  // Top right front
            Vec3::new(half_width, -half_height, -half_depth), // Bottom right back
            Vec3::new(half_width, half_height, -half_depth), // Top right back
            // Back face
            Vec3::new(half_width, -half_height, -half_depth), // Bottom right back
            Vec3::new(-half_width, -half_height, -half_depth), // Bottom left back
            Vec3::new(half_width, half_height, -half_depth),  // Top right back
            Vec3::new(half_width, half_height, -half_depth),  // Top right back
            Vec3::new(-half_width, -half_height, -half_depth), // Bottom left back
            Vec3::new(-half_width, half_height, -half_depth), // Top left back
            // Bottom face
            Vec3::new(half_width, -half_height, half_depth), // Bottom right front
            Vec3::new(-half_width, -half_height, half_depth), // Bottom left front
            Vec3::new(half_width, -half_height, -half_depth), // Bottom right back
            Vec3::new(half_width, -half_height, -half_depth), // Bottom right back
            Vec3::new(-half_width, -half_height, half_depth), // Bottom left front
            Vec3::new(-half_width, -half_height, -half_depth), // Bottom left back
            // Top face
            Vec3::new(-half_width, half_height, half_depth), // Top left front
            Vec3::new(half_width, half_height, half_depth),  // Top right front
            Vec3::new(-half_width, half_height, -half_depth), // Top left back
            Vec3::new(-half_width, half_height, -half_depth), // Top left back
            Vec3::new(half_width, half_height, half_depth),  // Top right front
            Vec3::new(half_width, half_height, -half_depth), // Top right back
        ];

        let indices = [
            0, 1, 2, 3, 4, 5, // Front face
            6, 7, 8, 9, 10, 11, // Left face
            12, 13, 14, 15, 16, 17, // Right face
            18, 19, 20, 21, 22, 23, // Back face
            24, 25, 26, 27, 28, 29, // Bottom face
            30, 31, 32, 33, 34, 35, // Top face
        ];

        let normals = [
            // Front face
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
            // Left face
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(-1.0, 0.0, 0.0),
            // Right face
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            // Back face
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 0.0, -1.0),
            // Bottom face
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            // Top face
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];

        Mesh::new(&vertices, &indices, &normals, &[Vec2::zeros(), Vec2::zeros(), Vec2::zeros()])
    }
}
