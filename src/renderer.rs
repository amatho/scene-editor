use std::sync::Arc;

use bevy_ecs::prelude::*;
use glow::{Context, HasContext};
use nalgebra_glm as glm;

use crate::components::{
    CustomShader, CustomTexture, Mesh, PointLight, Position, Rotation, Scale, Selected, StencilId,
};
use crate::gl_util;
use crate::resources::{Camera, RenderState, WinitWindow};

type GeometryQuery<'a> = (
    Entity,
    &'a Mesh,
    &'a Position,
    &'a Rotation,
    &'a Scale,
    Option<&'a Selected>,
    Option<&'a CustomShader>,
    Option<&'a CustomTexture>,
);

pub fn render(
    gl: NonSend<Arc<Context>>,
    camera: Res<Camera>,
    render_state: Res<RenderState>,
    window: Res<WinitWindow>,
    geometry: Query<GeometryQuery>,
    lights: Query<(&PointLight, &Position)>,
    mut commands: Commands,
) {
    let window_size = window.inner_size();

    let light_space_matrix = glm::ortho(-15.0f32, 15.0, -10.0, 10.0, -15.0, 15.0)
        * glm::look_at(
            &glm::vec3(0.2, 0.7, 0.5),
            &glm::vec3(0.0, 0.0, 0.0),
            &glm::vec3(0.0, 1.0, 0.0),
        );

    render_state.depth_shader.activate(&gl);

    unsafe {
        // Fix after egui_glow and prepare for shadow mapping
        gl.enable(glow::DEPTH_TEST);
        gl.depth_func(glow::LESS);
        gl.enable(glow::CULL_FACE);
        gl.cull_face(glow::BACK);

        gl_util::uniform_mat4(
            &gl,
            render_state.depth_shader.program,
            "lightSpaceMatrix",
            &light_space_matrix,
        );

        let (width, height) = render_state.shadow_map_size;
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(render_state.shadow_map_fbo));
        gl.viewport(0, 0, width, height);
        gl.clear(glow::DEPTH_BUFFER_BIT);
    }

    for (_, mesh, &pos, &rot, &scale, _, _, _) in &geometry {
        let model = glm::translation(&pos.into())
            * glm::rotation(rot.y.to_radians(), &glm::vec3(0.0, 1.0, 0.0))
            * glm::rotation(rot.x.to_radians(), &glm::vec3(1.0, 0.0, 0.0))
            * glm::rotation(rot.z.to_radians(), &glm::vec3(0.0, 0.0, 1.0))
            * glm::scaling(&scale.into());

        unsafe {
            gl_util::uniform_mat4(&gl, render_state.depth_shader.program, "model", &model);
            gl.bind_vertex_array(Some(mesh.vao_id));
            gl.draw_elements(glow::TRIANGLES, mesh.indices_len as i32, glow::UNSIGNED_INT, 0);
        }
    }

    // Geometry pass
    unsafe {
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(render_state.g_buffer));
        gl.viewport(0, 0, window_size.width as i32, window_size.height as i32);

        gl.clear_color(0.0, 0.0, 0.0, 0.0);
        gl.stencil_mask(0xFF);
        gl.clear_stencil(0);
        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);

        gl.disable(glow::BLEND);

        gl.enable(glow::STENCIL_TEST);
        gl.stencil_op(glow::KEEP, glow::KEEP, glow::REPLACE);
    }

    let vp =
        camera.projection * glm::look_at(&camera.pos, &(camera.pos + camera.front), &camera.up);

    for (i, (entity, mesh, &pos, &rot, &scale, selected, custom_shader, custom_texture)) in
        geometry.iter().enumerate()
    {
        let model = glm::translation(&pos.into())
            * glm::rotation(rot.y.to_radians(), &glm::vec3(0.0, 1.0, 0.0))
            * glm::rotation(rot.x.to_radians(), &glm::vec3(1.0, 0.0, 0.0))
            * glm::rotation(rot.z.to_radians(), &glm::vec3(0.0, 0.0, 1.0))
            * glm::scaling(&scale.into());

        let mvp = vp * model;
        let normal_mat = glm::mat4_to_mat3(&model.try_inverse().unwrap().transpose());
        let id = i + 1;

        let shader = if let Some(CustomShader { shader: Ok(shader), .. }) = custom_shader {
            shader
        } else {
            &render_state.geometry_pass_shader
        };
        shader.activate(&gl);

        unsafe {
            let texture = custom_texture.copied().unwrap_or_default();
            let diffuse = texture.diffuse.unwrap_or(render_state.default_diffuse);
            let specular = texture.specular.unwrap_or(render_state.default_specular);
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(diffuse));
            gl.active_texture(glow::TEXTURE1);
            gl.bind_texture(glow::TEXTURE_2D, Some(specular));
            gl_util::uniform_int(&gl, shader.program, "diffuse", 0);
            gl_util::uniform_int(&gl, shader.program, "specular", 1);

            gl_util::uniform_mat4(&gl, shader.program, "mvp", &mvp);
            gl_util::uniform_mat4(&gl, shader.program, "model", &model);
            gl_util::uniform_mat3(&gl, shader.program, "normalMat", &normal_mat);
            gl_util::uniform_float(&gl, shader.program, "selected", 0.0);

            gl.stencil_func(glow::ALWAYS, id as i32, 0xFF);
            gl.bind_vertex_array(Some(mesh.vao_id));
            gl.draw_elements(glow::TRIANGLES, mesh.indices_len as i32, glow::UNSIGNED_INT, 0);

            if selected.is_some() {
                // Redraw the object in bigger scale, with stencil testing and outline
                let mvp = mvp
                    * glm::scaling(
                        &glm::Vec3::from(scale)
                            .add_scalar(0.1)
                            .component_div(&glm::Vec3::from(scale)),
                    );

                render_state.geometry_pass_shader.activate(&gl);
                gl_util::uniform_int(&gl, render_state.geometry_pass_shader.program, "diffuse", 0);
                gl_util::uniform_int(&gl, render_state.geometry_pass_shader.program, "specular", 1);

                gl_util::uniform_mat4(&gl, render_state.geometry_pass_shader.program, "mvp", &mvp);
                gl_util::uniform_mat4(
                    &gl,
                    render_state.geometry_pass_shader.program,
                    "model",
                    &model,
                );
                gl_util::uniform_mat3(
                    &gl,
                    render_state.geometry_pass_shader.program,
                    "normalMat",
                    &normal_mat,
                );
                gl_util::uniform_float(
                    &gl,
                    render_state.geometry_pass_shader.program,
                    "selected",
                    1.0,
                );

                // Disable writing to the stencil buffer
                gl.stencil_mask(0x00);
                // Pass if the fragment does not overlap with the object we're highlighting
                gl.stencil_func(glow::NOTEQUAL, id as i32, 0xFF);
                gl.draw_elements(glow::TRIANGLES, mesh.indices_len as i32, glow::UNSIGNED_INT, 0);
                // Re-enable writing to the stencil buffer
                gl.stencil_mask(0xFF);
            }
        }

        commands.entity(entity).insert(StencilId(id));
    }

    // Deferred lighting pass
    unsafe {
        // Disable stencil test to make sure the quad and UI are drawn correctly
        gl.disable(glow::STENCIL_TEST);

        gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        gl.viewport(0, 0, window_size.width as i32, window_size.height as i32);

        gl.clear_color(0.0, 0.0, 0.0, 0.0);
        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

        render_state.deferred_pass_shader.activate(&gl);

        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(render_state.g_position));
        gl.active_texture(glow::TEXTURE1);
        gl.bind_texture(glow::TEXTURE_2D, Some(render_state.g_normal));
        gl.active_texture(glow::TEXTURE2);
        gl.bind_texture(glow::TEXTURE_2D, Some(render_state.g_albedo_spec));
        gl.active_texture(glow::TEXTURE3);
        gl.bind_texture(glow::TEXTURE_2D, Some(render_state.shadow_map));

        gl_util::uniform_int(&gl, render_state.deferred_pass_shader.program, "gPosition", 0);
        gl_util::uniform_int(&gl, render_state.deferred_pass_shader.program, "gNormal", 1);
        gl_util::uniform_int(&gl, render_state.deferred_pass_shader.program, "gAlbedoSpec", 2);
        gl_util::uniform_vec3(
            &gl,
            render_state.deferred_pass_shader.program,
            "viewPos",
            &camera.pos,
        );

        gl_util::uniform_mat4(
            &gl,
            render_state.deferred_pass_shader.program,
            "lightSpaceMatrix",
            &light_space_matrix,
        );
        gl_util::uniform_int(&gl, render_state.deferred_pass_shader.program, "shadowMap", 3);

        // TODO: Make this configurable
        gl_util::uniform_vec3(
            &gl,
            render_state.deferred_pass_shader.program,
            "dirLight.direction",
            &glm::vec3(-0.2, -0.7, -0.5),
        );
        gl_util::uniform_vec3(
            &gl,
            render_state.deferred_pass_shader.program,
            "dirLight.ambient",
            &glm::vec3(0.2, 0.2, 0.2),
        );
        gl_util::uniform_vec3(
            &gl,
            render_state.deferred_pass_shader.program,
            "dirLight.diffuse",
            &glm::vec3(0.5, 0.5, 0.5),
        );
        gl_util::uniform_vec3(
            &gl,
            render_state.deferred_pass_shader.program,
            "dirLight.specular",
            &glm::vec3(1.0, 1.0, 1.0),
        );

        let lights_iter = lights.iter();
        let lights_len = lights_iter.len();
        for (i, (light, &light_pos)) in lights_iter.enumerate() {
            gl_util::uniform_vec3(
                &gl,
                render_state.deferred_pass_shader.program,
                &format!("pointLights[{i}].position"),
                &light_pos.into(),
            );
            gl_util::uniform_vec3(
                &gl,
                render_state.deferred_pass_shader.program,
                &format!("pointLights[{i}].ambient"),
                &light.ambient,
            );
            gl_util::uniform_vec3(
                &gl,
                render_state.deferred_pass_shader.program,
                &format!("pointLights[{i}].diffuse"),
                &light.diffuse,
            );
            gl_util::uniform_vec3(
                &gl,
                render_state.deferred_pass_shader.program,
                &format!("pointLights[{i}].specular"),
                &light.specular,
            );
            gl_util::uniform_float(
                &gl,
                render_state.deferred_pass_shader.program,
                &format!("pointLights[{i}].constant"),
                light.constant,
            );
            gl_util::uniform_float(
                &gl,
                render_state.deferred_pass_shader.program,
                &format!("pointLights[{i}].linear"),
                light.linear,
            );
            gl_util::uniform_float(
                &gl,
                render_state.deferred_pass_shader.program,
                &format!("pointLights[{i}].quadratic"),
                light.quadratic,
            );
        }

        gl_util::uniform_int(
            &gl,
            render_state.deferred_pass_shader.program,
            "pointLightsSize",
            lights_len as i32,
        );

        gl.bind_vertex_array(Some(render_state.quad_vao.vao_id));
        gl.draw_elements(
            glow::TRIANGLES,
            render_state.quad_vao.indices_len as i32,
            glow::UNSIGNED_INT,
            0,
        );
    }
}
