use std::ptr;

use bevy_ecs::system::{Query, Res};
use nalgebra_glm as glm;

use crate::components::{Mesh, Position, Rotation, Scale};
use crate::resources::{Camera, ShaderState};

pub fn render(
    camera: Res<Camera>,
    shader_state: Res<ShaderState>,
    query: Query<(&Mesh, &Position, &Rotation, &Scale)>,
) {
    let vp = camera.projection * camera.view;

    for (m, p, r, s) in &query {
        let model = glm::translation(&glm::vec3(p.x, p.y, p.z))
            * glm::rotation(r.y, &glm::vec3(0.0, 1.0, 0.0))
            * glm::rotation(r.x, &glm::vec3(1.0, 0.0, 0.0))
            * glm::rotation(r.z, &glm::vec3(0.0, 0.0, 1.0))
            * glm::scaling(&glm::vec3(s.x, s.y, s.z));

        let mvp = vp * model;

        unsafe {
            let mvp_location =
                gl::GetUniformLocation(shader_state.program_id, b"mvp\0".as_ptr().cast());
            gl::UniformMatrix4fv(mvp_location, 1, gl::FALSE, glm::value_ptr(&mvp).as_ptr());

            gl::BindVertexArray(m.vao_id);
            gl::DrawElements(gl::TRIANGLES, m.num_indices as i32, gl::UNSIGNED_INT, ptr::null());
        }
    }
}
