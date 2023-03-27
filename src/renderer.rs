use std::sync::Arc;

use bevy_ecs::prelude::*;
use glow::{Context, HasContext};
use nalgebra_glm as glm;

use crate::components::{Mesh, Position, Rotation, Scale, StencilId};
use crate::resources::{Camera, ShaderState};

pub fn render(
    gl: NonSend<Arc<Context>>,
    camera: Res<Camera>,
    shader_state: Res<ShaderState>,
    query: Query<(Entity, &Mesh, &Position, &Rotation, &Scale)>,
    mut commands: Commands,
) {
    unsafe {
        gl.enable(glow::DEPTH_TEST);
        gl.depth_func(glow::LESS);

        gl.enable(glow::CULL_FACE);

        gl.clear_color(0.4, 0.4, 1.0, 1.0);
        gl.stencil_mask(0xFF);
        gl.clear_stencil(0);
        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);

        gl.enable(glow::STENCIL_TEST);
        gl.stencil_op(glow::KEEP, glow::KEEP, glow::REPLACE);
    }

    shader_state.shader.activate(&gl);

    let vp =
        camera.projection * glm::look_at(&camera.pos, &(camera.pos + camera.front), &camera.up);

    for (i, (entity, mesh, pos, rot, scale)) in query.iter().enumerate() {
        let model = glm::translation(&glm::vec3(pos.x, pos.y, pos.z))
            * glm::rotation(rot.y, &glm::vec3(0.0, 1.0, 0.0))
            * glm::rotation(rot.x, &glm::vec3(1.0, 0.0, 0.0))
            * glm::rotation(rot.z, &glm::vec3(0.0, 0.0, 1.0))
            * glm::scaling(&glm::vec3(scale.x, scale.y, scale.z));

        let mvp = vp * model;
        let id = i + 1;

        unsafe {
            let mvp_location = gl.get_uniform_location(shader_state.shader.program, "mvp");
            gl.uniform_matrix_4_f32_slice(mvp_location.as_ref(), false, glm::value_ptr(&mvp));

            gl.stencil_func(glow::ALWAYS, id as i32, 0xFF);
            gl.bind_vertex_array(Some(mesh.vao));
            gl.draw_elements(glow::TRIANGLES, mesh.num_indices as i32, glow::UNSIGNED_INT, 0);
        }

        commands.entity(entity).insert(StencilId(id));
    }

    unsafe {
        gl.stencil_mask(0);
    }
}
