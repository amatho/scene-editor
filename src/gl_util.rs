use std::{mem, ptr};

use gl::types::{GLboolean, GLenum, GLint, GLsizei, GLsizeiptr, GLuint};
use nalgebra_glm::{Vec2, Vec3};

unsafe fn buffer_with_data<T>(target: GLenum, data: &[T]) -> GLuint {
    let mut buf_id = 0;
    gl::GenBuffers(1, &mut buf_id);
    gl::BindBuffer(target, buf_id);
    gl::BufferData(
        target,
        mem::size_of_val(data) as GLsizeiptr,
        data.as_ptr().cast(),
        gl::STATIC_DRAW,
    );

    buf_id
}

pub unsafe fn generate_attribute<T>(
    id: GLuint,
    elements_per_entry: GLint,
    data: &[T],
    normalize: bool,
) -> GLuint {
    let buf_id = buffer_with_data(gl::ARRAY_BUFFER, data);
    gl::VertexAttribPointer(
        id,
        elements_per_entry,
        gl::FLOAT,
        normalize as GLboolean,
        mem::size_of::<T>() as GLsizei,
        ptr::null(),
    );
    gl::EnableVertexAttribArray(id);

    buf_id
}

pub unsafe fn create_vao(
    vertices: &[Vec3],
    indices: &[GLuint],
    normals: &[Vec3],
    texture_coords: &[Vec2],
) -> GLuint {
    let mut vao_id = 0;
    gl::GenVertexArrays(1, &mut vao_id);
    gl::BindVertexArray(vao_id);

    generate_attribute(0, 3, vertices, false);

    generate_attribute(1, 3, normals, false);

    generate_attribute(2, 3, texture_coords, false);

    buffer_with_data(gl::ELEMENT_ARRAY_BUFFER, indices);

    vao_id
}
