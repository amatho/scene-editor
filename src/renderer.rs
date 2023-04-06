use std::sync::Arc;

use bevy_ecs::prelude::*;
use glow::{Context, HasContext};
use nalgebra_glm as glm;

use crate::components::{
    CustomShader, Mesh, PointLight, Position, Rotation, Scale, Selected, StencilId,
};
use crate::gl_util;
use crate::resources::{Camera, RenderSettings};

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
    render_settings: Res<RenderSettings>,
    geometry: Query<GeometryQuery>,
    lights: Query<(&PointLight, &Position)>,
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

        // TODO: Move this down to object rendering and support custom texture
        gl.bind_texture(glow::TEXTURE_2D, Some(render_settings.default_texture));
    }

    let vp =
        camera.projection * glm::look_at(&camera.pos, &(camera.pos + camera.front), &camera.up);

    for (i, (entity, mesh, &pos, &rot, &scale, selected, custom_shader)) in
        geometry.iter().enumerate()
    {
        let model = glm::translation(&pos.into())
            * glm::rotation(rot.y.to_radians(), &glm::vec3(0.0, 1.0, 0.0))
            * glm::rotation(rot.x.to_radians(), &glm::vec3(1.0, 0.0, 0.0))
            * glm::rotation(rot.z.to_radians(), &glm::vec3(0.0, 0.0, 1.0))
            * glm::scaling(&scale.into());

        let mvp = vp * model;
        let id = i + 1;

        unsafe {
            let shader = if let Some(CustomShader { shader: Ok(shader), .. }) = custom_shader {
                shader
            } else {
                &render_settings.default_shader
            };

            shader.activate(&gl);
            gl_util::uniform_mat4(&gl, shader.program, "mvp", &mvp);
            gl_util::uniform_mat4(&gl, shader.program, "model", &model);

            let (light, &light_pos) = lights.single();
            gl_util::uniform_vec3(&gl, shader.program, "lightPos", &light_pos.into());
            gl_util::uniform_vec3(&gl, shader.program, "lightColor", &light.color);
            gl_util::uniform_vec3(&gl, shader.program, "viewPos", &camera.pos);

            gl.stencil_func(glow::ALWAYS, id as i32, 0xFF);
            gl.bind_vertex_array(Some(mesh.vao));
            gl.draw_elements(glow::TRIANGLES, mesh.num_indices as i32, glow::UNSIGNED_INT, 0);

            if selected.is_some() {
                // Redraw the object in bigger scale, with stencil testing and outline shader

                let mvp = mvp
                    * glm::scaling(
                        &glm::Vec3::from(scale)
                            .add_scalar(0.1)
                            .component_div(&glm::Vec3::from(scale)),
                    );

                render_settings.outline_shader.activate(&gl);

                gl_util::uniform_mat4(&gl, render_settings.outline_shader.program, "mvp", &mvp);

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
