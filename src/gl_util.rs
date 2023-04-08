use std::mem;

use bytemuck::Pod;
use glow::{Buffer, Context, HasContext, Program, VertexArray};
use nalgebra_glm as glm;
use tracing::warn;

#[derive(Clone)]
pub struct VertexArrayObject {
    pub vao_id: VertexArray,
    pub indices_len: usize,
    buffers: Box<[Buffer]>,
    destroyed: bool,
}

impl VertexArrayObject {
    pub unsafe fn new(
        gl: &Context,
        vertices: &[glm::Vec3],
        indices: &[u32],
        normals: &[glm::Vec3],
        texture_coords: &[glm::Vec2],
    ) -> Self {
        let vao_id = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao_id));

        let vert_buf = generate_attribute(gl, 0, 3, vertices, false);
        let normal_buf = generate_attribute(gl, 1, 3, normals, false);
        let tex_buf = generate_attribute(gl, 2, 2, texture_coords, false);
        let indices_buf = buffer_with_data(gl, glow::ELEMENT_ARRAY_BUFFER, indices);

        let indices_len = indices.len();
        let buffers = Box::new([vert_buf, normal_buf, tex_buf, indices_buf]);
        Self { vao_id, indices_len, buffers, destroyed: false }
    }

    /// # Safety
    ///
    /// The VAO and buffers are no longer valid and should not be used.
    pub unsafe fn destroy(&mut self, gl: &Context) {
        for buf in self.buffers.iter() {
            gl.delete_buffer(*buf);
        }
        gl.delete_vertex_array(self.vao_id);

        self.destroyed = true;
    }
}

impl Drop for VertexArrayObject {
    fn drop(&mut self) {
        if !self.destroyed {
            warn!("vertex array object was not destroyed (VAO: {:?})", self.vao_id);
        }
    }
}

unsafe fn buffer_with_data<T: Pod>(gl: &Context, target: u32, data: &[T]) -> Buffer {
    let buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(target, Some(buffer));
    gl.buffer_data_u8_slice(target, bytemuck::cast_slice(data), glow::STATIC_DRAW);

    buffer
}

pub unsafe fn generate_attribute<T: Pod>(
    gl: &Context,
    id: u32,
    elements_per_entry: i32,
    data: &[T],
    normalize: bool,
) -> Buffer {
    let buffer = buffer_with_data(gl, glow::ARRAY_BUFFER, data);
    gl.vertex_attrib_pointer_f32(
        id,
        elements_per_entry,
        glow::FLOAT,
        normalize,
        mem::size_of::<T>() as i32,
        0,
    );
    gl.enable_vertex_attrib_array(id);

    buffer
}

pub unsafe fn uniform_vec3(gl: &Context, program: Program, name: &str, value: &glm::Vec3) {
    let loc = gl.get_uniform_location(program, name);
    gl.uniform_3_f32_slice(loc.as_ref(), glm::value_ptr(value));
}

pub unsafe fn uniform_mat4(gl: &Context, program: Program, name: &str, value: &glm::Mat4) {
    let loc = gl.get_uniform_location(program, name);
    gl.uniform_matrix_4_f32_slice(loc.as_ref(), false, glm::value_ptr(value));
}

pub unsafe fn uniform_float(gl: &Context, program: Program, name: &str, value: f32) {
    let loc = gl.get_uniform_location(program, name);
    gl.uniform_1_f32(loc.as_ref(), value);
}

pub unsafe fn uniform_int(gl: &Context, program: Program, name: &str, value: i32) {
    let loc = gl.get_uniform_location(program, name);
    gl.uniform_1_i32(loc.as_ref(), value);
}
