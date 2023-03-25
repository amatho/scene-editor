use bevy_ecs::system::{Query, Res};
use glow::HasContext;
use nalgebra_glm as glm;

use crate::components::{Mesh, Position, Rotation, Scale};
use crate::resources::{Camera, GlContext, ShaderState};

pub fn render(
    gl_context: Res<GlContext>,
    camera: Res<Camera>,
    shader_state: Res<ShaderState>,
    query: Query<(&Mesh, &Position, &Rotation, &Scale)>,
) {
    let gl = &gl_context.gl;
    unsafe {
        gl.enable(glow::DEPTH_TEST);
        gl.depth_func(glow::LESS);

        gl.enable(glow::CULL_FACE);

        gl.clear_color(0.4, 0.4, 1.0, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
    }

    shader_state.shader.activate(gl);

    let vp =
        camera.projection * glm::look_at(&camera.pos, &(camera.pos + camera.front), &camera.up);

    for (m, p, r, s) in &query {
        let model = glm::translation(&glm::vec3(p.x, p.y, p.z))
            * glm::rotation(r.y, &glm::vec3(0.0, 1.0, 0.0))
            * glm::rotation(r.x, &glm::vec3(1.0, 0.0, 0.0))
            * glm::rotation(r.z, &glm::vec3(0.0, 0.0, 1.0))
            * glm::scaling(&glm::vec3(s.x, s.y, s.z));

        let mvp = vp * model;

        unsafe {
            let mvp_location = gl.get_uniform_location(shader_state.shader.program, "mvp");
            gl.uniform_matrix_4_f32_slice(mvp_location.as_ref(), false, glm::value_ptr(&mvp));

            gl.bind_vertex_array(Some(m.vao));
            gl.draw_elements(glow::TRIANGLES, m.num_indices as i32, glow::UNSIGNED_INT, 0);
        }
    }
}
