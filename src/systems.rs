use bevy_ecs::system::{Query, Res, ResMut};
use nalgebra_glm as glm;
use winit::event::VirtualKeyCode;

use crate::components::Rotation;
use crate::resources::{Camera, Input, Time};

pub fn move_camera(mut input: ResMut<Input>, mut camera: ResMut<Camera>, time: Res<Time>) {
    let front = camera.front;
    let up = camera.up;
    const CAMERA_SPEED: f32 = 25.0;
    const CAMERA_SENSITIVITY: f32 = 0.1;

    let speed_modifier = if input.is_pressed(VirtualKeyCode::LShift) { 3.0 } else { 1.0 };

    camera.yaw += input.mouse_delta.0 as f32 * CAMERA_SENSITIVITY;
    camera.pitch -= input.mouse_delta.1 as f32 * CAMERA_SENSITIVITY;

    camera.front = glm::normalize(&glm::vec3(
        camera.yaw.to_radians().cos() * camera.pitch.to_radians().cos(),
        camera.pitch.to_radians().sin(),
        camera.yaw.to_radians().sin() * camera.pitch.to_radians().cos(),
    ));

    input.mouse_delta = (0.0, 0.0);

    let speed = CAMERA_SPEED * time.delta_time * speed_modifier;
    if input.is_pressed(VirtualKeyCode::W) {
        camera.pos += speed * front;
    }
    if input.is_pressed(VirtualKeyCode::S) {
        camera.pos -= speed * front;
    }
    if input.is_pressed(VirtualKeyCode::A) {
        camera.pos -= speed * glm::normalize(&glm::cross(&front, &up));
    }
    if input.is_pressed(VirtualKeyCode::D) {
        camera.pos += speed * glm::normalize(&glm::cross(&front, &up));
    }
    if input.is_pressed(VirtualKeyCode::Space) {
        camera.pos += speed * up;
    }
    if input.is_pressed(VirtualKeyCode::LControl) {
        camera.pos -= speed * up;
    }
}

// TODO: Remove
pub fn rotate_objects(time: Res<Time>, mut query: Query<&mut Rotation>) {
    for mut r in query.iter_mut() {
        r.x += time.delta_time;
        r.y += time.delta_time;
        r.z += time.delta_time;
    }
}
