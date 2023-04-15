use std::sync::Arc;

use bevy_ecs::prelude::*;
use glow::{Context, HasContext, PixelPackData};
use nalgebra_glm as glm;
use tracing::debug;
use winit::event::{MouseButton, VirtualKeyCode};

use crate::components::{Mesh, Position, Selected, StencilId, TransformBundle};
use crate::resources::{Camera, Input, ModelLoader, RenderState, Time, WinitWindow};

pub fn move_camera(input: Res<Input>, mut camera: ResMut<Camera>, time: Res<Time>) {
    let front = camera.front;
    let up = camera.up;
    const CAMERA_SPEED: f32 = 10.0;
    const CAMERA_SENSITIVITY: f64 = 0.3;

    let speed_modifier =
        if input.get_key_press_continuous(VirtualKeyCode::LShift) { 3.0 } else { 1.0 };

    camera.yaw += input.mouse_delta.0 * CAMERA_SENSITIVITY;
    camera.pitch -= input.mouse_delta.1 * CAMERA_SENSITIVITY;
    camera.pitch = camera.pitch.clamp(-89.0, 89.0);

    let yaw_radians = camera.yaw.to_radians();
    let pitch_radians = camera.pitch.to_radians();
    camera.front = glm::normalize(&glm::vec3(
        (yaw_radians.cos() * pitch_radians.cos()) as f32,
        pitch_radians.sin() as f32,
        (yaw_radians.sin() * pitch_radians.cos()) as f32,
    ));

    let speed = CAMERA_SPEED * time.delta_seconds() * speed_modifier;
    if input.get_key_press_continuous(VirtualKeyCode::W) {
        camera.pos += speed * front;
    }
    if input.get_key_press_continuous(VirtualKeyCode::S) {
        camera.pos -= speed * front;
    }
    if input.get_key_press_continuous(VirtualKeyCode::A) {
        camera.pos -= speed * glm::normalize(&glm::cross(&front, &up));
    }
    if input.get_key_press_continuous(VirtualKeyCode::D) {
        camera.pos += speed * glm::normalize(&glm::cross(&front, &up));
    }
    if input.get_key_press_continuous(VirtualKeyCode::Space) {
        camera.pos += speed * up;
    }
    if input.get_key_press_continuous(VirtualKeyCode::LControl) {
        camera.pos -= speed * up;
    }
}

pub fn spawn_object(
    camera: Res<Camera>,
    input: Res<Input>,
    model_loader: Res<ModelLoader>,
    mut commands: Commands,
) {
    if input.get_key_press(VirtualKeyCode::E) {
        let spawn_pos = camera.pos + camera.front * 3.0;
        let position = Position::new(spawn_pos.x, spawn_pos.y, spawn_pos.z);

        debug!("spawning a cube at {:?}", position);

        let mesh = Mesh::from(model_loader.get("Cube").unwrap());
        commands.spawn((mesh, TransformBundle { position, ..Default::default() }));
    }
}

pub fn select_object(
    gl: NonSend<Arc<Context>>,
    window: Res<WinitWindow>,
    input: Res<Input>,
    render_state: Res<RenderState>,
    already_selected: Query<Entity, With<Selected>>,
    query: Query<(Entity, &StencilId)>,
    mut commands: Commands,
) {
    if input.get_mouse_button_press(MouseButton::Left) {
        for entity in &already_selected {
            commands.entity(entity).remove::<Selected>();
        }

        let (x, y) = input.mouse_pos;
        let window_height = window.inner_size().height;
        let index = unsafe {
            let mut bytes = [0; 4];
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(render_state.g_buffer));
            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(render_state.g_rbo));
            gl.read_pixels(
                x as i32,
                window_height as i32 - y as i32 - 1,
                1,
                1,
                glow::DEPTH_STENCIL,
                glow::UNSIGNED_INT_24_8,
                PixelPackData::Slice(&mut bytes),
            );
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            let pixel = u32::from_ne_bytes(bytes);
            (pixel & 0xFF) as usize
        };

        let mut found = false;
        for (entity, stencil_id) in &query {
            if stencil_id.0 == index {
                commands.entity(entity).insert(Selected);
                found = true;
                debug!("selected entity {}", entity.index());
                break;
            }
        }

        if !found {
            debug!("found no object to select");
        }
    }
}
