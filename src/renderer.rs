use std::sync::Arc;

use bevy_ecs::prelude::*;
use glow::{Context, HasContext};
use nalgebra_glm as glm;

use crate::components::{CustomShader, Mesh, Position, Rotation, Scale, Selected, StencilId};
use crate::resources::{Camera, DefaultShader};

type GeometryQuery<'a> = (
    Entity,
    &'a Mesh,
    &'a Position,
    &'a Rotation,
    &'a Scale,
    Option<&'a Selected>,
    Option<&'a CustomShader>,
);

pub fn render(
    gl: NonSend<Arc<Context>>,
    camera: Res<Camera>,
    shader_state: Res<DefaultShader>,
    query: Query<GeometryQuery>,
    mut commands: Commands,
) {
    unsafe {
        // Enable various features.
        // Some are disabled by egui_glow, and need to be enabled each time we render.
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

    let vp =
        camera.projection * glm::look_at(&camera.pos, &(camera.pos + camera.front), &camera.up);

    for (i, (entity, mesh, pos, rot, scale, selected, custom_shader)) in query.iter().enumerate() {
        let model = glm::translation(&glm::vec3(pos.x, pos.y, pos.z))
            * glm::rotation(rot.y, &glm::vec3(0.0, 1.0, 0.0))
            * glm::rotation(rot.x, &glm::vec3(1.0, 0.0, 0.0))
            * glm::rotation(rot.z, &glm::vec3(0.0, 0.0, 1.0))
            * glm::scaling(&glm::vec3(scale.x, scale.y, scale.z));

        let mvp = vp * model;
        let id = i + 1;

        unsafe {
            if let Some(CustomShader { shader: Ok(shader), .. }) = custom_shader {
                shader.activate(&gl);
            } else {
                shader_state.shader.activate(&gl);
            }

            let mvp_location = gl.get_uniform_location(shader_state.shader.program, "mvp");
            gl.uniform_matrix_4_f32_slice(mvp_location.as_ref(), false, glm::value_ptr(&mvp));

            gl.stencil_func(glow::ALWAYS, id as i32, 0xFF);
            gl.bind_vertex_array(Some(mesh.vao));
            gl.draw_elements(glow::TRIANGLES, mesh.num_indices as i32, glow::UNSIGNED_INT, 0);

            if selected.is_some() {
                // Redraw the object in bigger scale, with stencil testing and outline shader

                let mvp = mvp * glm::scaling(&glm::vec3(1.1, 1.1, 1.1));

                shader_state.outline.activate(&gl);

                let mvp_location = gl.get_uniform_location(shader_state.shader.program, "mvp");
                gl.uniform_matrix_4_f32_slice(mvp_location.as_ref(), false, glm::value_ptr(&mvp));

                // Disable writing to the stencil buffer
                gl.stencil_mask(0x00);
                // Pass if the fragment does not overlap with the object we're highlighting
                gl.stencil_func(glow::NOTEQUAL, id as i32, 0xFF);
                gl.draw_elements(glow::TRIANGLES, mesh.num_indices as i32, glow::UNSIGNED_INT, 0);
                // Re-enable writing to the stencil buffer
                gl.stencil_mask(0xFF);
            }
        }

        commands.entity(entity).insert(StencilId(id));
    }

    unsafe {
        // Disable stencil test to make sure UI is drawn correctly
        gl.disable(glow::STENCIL_TEST);
    }
}
