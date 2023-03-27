use std::mem;

use bytemuck::Pod;
use glow::{Buffer, Context, HasContext, VertexArray};
use nalgebra_glm as glm;

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

pub unsafe fn create_vao(
    gl: &Context,
    vertices: &[glm::Vec3],
    indices: &[u32],
    normals: &[glm::Vec3],
    texture_coords: &[glm::Vec2],
) -> (VertexArray, [Buffer; 4]) {
    let vao = gl.create_vertex_array().unwrap();
    gl.bind_vertex_array(Some(vao));

    let vert_buf = generate_attribute(gl, 0, 3, vertices, false);

    let normal_buf = generate_attribute(gl, 1, 3, normals, false);

    let tex_buf = generate_attribute(gl, 2, 2, texture_coords, false);

    let indices_buf = buffer_with_data(gl, glow::ELEMENT_ARRAY_BUFFER, indices);

    (vao, [vert_buf, normal_buf, tex_buf, indices_buf])
}

pub unsafe fn delete_vao(gl: &Context, vao: VertexArray, buffers: &[Buffer]) {
    gl.bind_vertex_array(Some(vao));
    for buf in buffers {
        gl.delete_buffer(*buf);
    }
    gl.delete_vertex_array(vao);
}
