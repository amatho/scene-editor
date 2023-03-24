mod components;
mod debug;
mod gl_util;
mod renderer;
mod resources;
mod shader;
mod systems;

use std::borrow::Cow;
use std::ffi::CString;
use std::time::Instant;

use bevy_ecs::schedule::{ExecutorKind, Schedule};
use bevy_ecs::world::World;
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, GlProfile, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin_winit::{DisplayBuilder, GlWindow};
use nalgebra_glm as glm;
use raw_window_handle::HasRawWindowHandle;
use winit::event::{DeviceEvent, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::window::{CursorGrabMode, WindowBuilder};

use crate::components::{Mesh, Position, Rotation, TransformBundle};
use crate::resources::{Camera, Input, ShaderState, Time};
use crate::shader::{ShaderBuilder, ShaderType};

pub fn run() -> Result<(), Cow<'static, str>> {
    let event_loop = winit::event_loop::EventLoop::new();
    let window_builder = WindowBuilder::new();
    let template = ConfigTemplateBuilder::new();
    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

    let (window, gl_config) = display_builder
        .build(&event_loop, template, |configs| {
            configs
                .reduce(|acc, cfg| if cfg.num_samples() > acc.num_samples() { cfg } else { acc })
                .unwrap()
        })
        .unwrap();

    println!("Picked a config with {} samples", gl_config.num_samples());

    let window = window.unwrap();
    window
        .set_cursor_grab(CursorGrabMode::Confined)
        .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked))
        .unwrap();
    window.set_cursor_visible(false);
    let raw_window_handle = window.raw_window_handle();

    let gl_display = gl_config.display();

    let context_attributes = ContextAttributesBuilder::new()
        .with_profile(GlProfile::Core)
        .with_context_api(ContextApi::OpenGl(Some(Version::new(4, 1)))) // Maximum supported version on macOS
        .build(Some(raw_window_handle));
    let not_current_gl_context =
        unsafe { gl_display.create_context(&gl_config, &context_attributes).unwrap() };

    let attrs = window.build_surface_attributes(Default::default());
    let gl_surface =
        unsafe { gl_config.display().create_window_surface(&gl_config, &attrs).unwrap() };

    let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

    gl::load_with(|s| {
        let s = CString::new(s).unwrap();
        gl_display.get_proc_address(s.as_c_str())
    });

    debug::print_gl_info();

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);

        gl::Enable(gl::CULL_FACE);

        gl::Disable(gl::DITHER);

        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        gl::ClearColor(0.4, 0.4, 1.0, 1.0);
    }

    let shader = ShaderBuilder::new()
        .add_shader("shaders/simple.vert", ShaderType::Vertex)?
        .add_shader("shaders/simple.frag", ShaderType::Fragment)?
        .link()?;
    shader.activate();

    let mut world = World::new();
    world.spawn((
        Mesh::cube(5.0, 5.0, 5.0),
        TransformBundle {
            position: Position::new(5.0, 0.0, -15.0),
            rotation: Rotation::new(0.0, 0.0, 0.0),
            ..Default::default()
        },
    ));
    world.spawn((
        Mesh::cube(5.0, 5.0, 5.0),
        TransformBundle {
            position: Position::new(-5.0, 0.0, -15.0),
            rotation: Rotation::new(0.0, 0.0, 0.0),
            ..Default::default()
        },
    ));

    let window_size = window.inner_size();
    let perspective = glm::perspective(
        80.0_f32.to_radians(),
        window_size.width as f32 / window_size.height as f32,
        0.1,
        350.0,
    );
    world.insert_resource(Camera::new(
        perspective,
        glm::vec3(0.0, 0.0, 3.0),
        glm::vec3(0.0, 0.0, -1.0),
        glm::vec3(0.0, 1.0, 0.0),
        -90.0,
        0.0,
    ));
    world.insert_resource(ShaderState::new(shader.program_id));
    world.insert_resource(Input::default());
    world.insert_resource(Time::default());

    let mut schedule = Schedule::default();
    schedule.add_system(systems::move_camera);
    schedule.add_system(systems::rotate_objects);

    let mut render_schedule = Schedule::new();
    render_schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    render_schedule.add_system(renderer::render);

    let mut previous_frame_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        let now = Instant::now();
        let delta_time = now.duration_since(previous_frame_time).as_secs_f32();
        previous_frame_time = now;
        world.resource_mut::<Time>().delta_time = delta_time;

        control_flow.set_poll();

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input: KeyboardInput { state, virtual_keycode: Some(keycode), .. },
                    ..
                } => match keycode {
                    VirtualKeyCode::Escape => control_flow.set_exit(),
                    k => world.resource_mut::<Input>().handle_keyboard_input(k, state),
                },
                WindowEvent::Resized(size) => {
                    if size.width != 0 && size.height != 0 {
                        let perspective = glm::perspective(
                            80.0_f32.to_radians(),
                            size.width as f32 / size.height as f32,
                            0.1,
                            350.0,
                        );
                        world.resource_mut::<Camera>().projection = perspective;

                        gl_surface.resize(
                            &gl_context,
                            size.width.try_into().unwrap(),
                            size.height.try_into().unwrap(),
                        );
                    }
                }
                WindowEvent::CloseRequested => {
                    control_flow.set_exit();
                }
                _ => (),
            },
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                world.resource_mut::<Input>().mouse_delta = delta;
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawEventsCleared => unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                schedule.run(&mut world);
                render_schedule.run(&mut world);

                gl_surface.swap_buffers(&gl_context).unwrap();
            },
            _ => (),
        }
    });
}
