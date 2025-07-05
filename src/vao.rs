use std::mem;

use bytemuck::Pod;
use glow::{Buffer, Context, HasContext, VertexArray};
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
        unsafe {
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
    }

    /// # Safety
    ///
    /// The VAO and buffers are no longer valid and should not be used.
    pub unsafe fn destroy(&mut self, gl: &Context) {
        unsafe {
            for buf in self.buffers.iter() {
                gl.delete_buffer(*buf);
            }
            gl.delete_vertex_array(self.vao_id);

            self.destroyed = true;
        }
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
    unsafe {
        let buffer = gl.create_buffer().unwrap();
        gl.bind_buffer(target, Some(buffer));
        gl.buffer_data_u8_slice(target, bytemuck::cast_slice(data), glow::STATIC_DRAW);

        buffer
    }
}

pub unsafe fn generate_attribute<T: Pod>(
    gl: &Context,
    id: u32,
    elements_per_entry: i32,
    data: &[T],
    normalize: bool,
) -> Buffer {
    unsafe {
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
}
