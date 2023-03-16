use std::ptr;

use bevy_ecs::system::Query;

use crate::components::{Mesh, Position, Rotation, Scale};

pub fn render(query: Query<(&Mesh, &Position, &Rotation, &Scale)>) {
    for (m, _p, _r, _s) in &query {
        unsafe {
            gl::BindVertexArray(m.vao_id);
            gl::DrawElements(gl::TRIANGLES, m.num_indices as i32, gl::UNSIGNED_INT, ptr::null());
        }
    }
}
