use std::sync::Arc;

use bevy_ecs::prelude::*;
use glow::{Context, HasContext, PixelPackData};
use nalgebra_glm as glm;
use tracing::debug;
use winit::event::{MouseButton, VirtualKeyCode};

use crate::commands;
use crate::components::{Position, Selected, StencilId, TransformBundle, UnloadedMesh};
use crate::resources::{Camera, Input, ModelLoader, Time, UiState};

pub fn move_camera(mut input: ResMut<Input>, mut camera: ResMut<Camera>, time: Res<Time>) {
    if !input.is_changed() {
        return;
    }

    let front = camera.front;
    let up = camera.up;
    const CAMERA_SPEED: f32 = 25.0;
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

    input.mouse_delta = (0.0, 0.0);

    let speed = CAMERA_SPEED * time.delta_time * speed_modifier;
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
    mut input: ResMut<Input>,
    ui_state: Res<UiState>,
    model_loader: ResMut<ModelLoader>,
    mut commands: Commands,
) {
    if (ui_state.camera_focused && input.get_mouse_button_press(MouseButton::Left))
        || input.get_key_press(VirtualKeyCode::E)
    {
        let spawn_pos = camera.pos + camera.front * 3.0;
        let position = Position::new(spawn_pos.x, spawn_pos.y, spawn_pos.z);

        debug!("spawning a cube at {:?}", position);

        commands.spawn((
            UnloadedMesh::from(model_loader.get("cube.obj").unwrap()),
            TransformBundle { position, ..Default::default() },
        ));
    }
}

pub fn select_object(
    gl: NonSend<Arc<Context>>,
    ui_state: Res<UiState>,
    mut input: ResMut<Input>,
    already_selected: Query<(Entity, &Selected)>,
    query: Query<(Entity, &StencilId)>,
    mut commands: Commands,
) {
    if !ui_state.camera_focused && input.get_mouse_button_press(MouseButton::Left) {
        for (entity, _) in &already_selected {
            commands.entity(entity).remove::<Selected>();
        }

        let (x, y) = input.mouse_pos;
        let index = unsafe {
            let mut bytes = [0; 4];
            gl.read_pixels(
                x as i32,
                ui_state.height as i32 - y as i32 - 1,
                1,
                1,
                glow::STENCIL_INDEX,
                glow::UNSIGNED_INT,
                PixelPackData::Slice(&mut bytes),
            );
            u32::from_ne_bytes(bytes) as usize
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

pub fn load_object_meshes(query: Query<Entity, Added<UnloadedMesh>>, mut commands: Commands) {
    for entity in &query {
        commands.entity(entity).add(commands::load_mesh);
    }
}
