mod components;
mod gl_util;
mod renderer;
mod resources;
mod shader;
mod systems;
mod ui;

use std::borrow::Cow;
use std::ffi::CString;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Instant;

use bevy_ecs::schedule::{ExecutorKind, IntoSystemConfigs, Schedule};
use bevy_ecs::world::World;
use egui_glow::EguiGlow;
use env_logger::Env;
use glow::HasContext as _;
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, GlProfile, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::SwapInterval;
use glutin_winit::{DisplayBuilder, GlWindow};
use log::info;
use nalgebra_glm as glm;
use raw_window_handle::HasRawWindowHandle;
use winit::event::{
    DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent,
};
use winit::window::{CursorGrabMode, WindowBuilder};

use crate::components::{Mesh, Position, Rotation, TransformBundle};
use crate::resources::{Camera, EguiGlowRes, Input, ShaderState, Time, WindowState, WinitWindow};
use crate::shader::{ShaderBuilder, ShaderType};

pub fn run() -> Result<(), Cow<'static, str>> {
    env_logger::Builder::from_env(Env::default().default_filter_or(if cfg!(debug_assertions) {
        "debug"
    } else {
        "warn"
    }))
    .init();

    let event_loop = winit::event_loop::EventLoop::new();
    let window_builder = WindowBuilder::new();
    let template = ConfigTemplateBuilder::new().with_stencil_size(8);
    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

    let (window, gl_config) = display_builder
        .build(&event_loop, template, |configs| {
            configs
                .reduce(|acc, cfg| if cfg.num_samples() > acc.num_samples() { cfg } else { acc })
                .unwrap()
        })
        .unwrap();

    info!("Picked a config with {} samples", gl_config.num_samples());
    info!("Picked a config with {} stencil size", gl_config.stencil_size());

    let window = window.unwrap();
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

    gl_surface
        .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
        .unwrap();

    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            let s = CString::new(s).expect("failed to construct C string for gl proc address");
            gl_display.get_proc_address(&s)
        })
    };
    let gl = Arc::new(gl);

    unsafe {
        info!("Vendor: {}", gl.get_parameter_string(glow::VENDOR));
        info!("Renderer: {}", gl.get_parameter_string(glow::RENDERER));
        info!("OpenGL Version: {}", gl.get_parameter_string(glow::VERSION));
        info!("GLSL Version: {}", gl.get_parameter_string(glow::SHADING_LANGUAGE_VERSION));

        gl.enable(glow::BLEND);
        gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
    }

    let mut world = World::new();
    world.spawn((
        Mesh::cube(&gl, 5.0, 5.0, 5.0),
        TransformBundle {
            position: Position::new(5.0, 0.0, -15.0),
            rotation: Rotation::new(0.0, 0.0, 0.0),
            ..Default::default()
        },
    ));
    world.spawn((
        Mesh::cube(&gl, 5.0, 5.0, 5.0),
        TransformBundle {
            position: Position::new(-5.0, 0.0, -15.0),
            rotation: Rotation::new(0.0, 0.0, 0.0),
            ..Default::default()
        },
    ));

    let shader = ShaderBuilder::new(&gl)
        .add_shader("shaders/simple.vert", ShaderType::Vertex)?
        .add_shader("shaders/simple.frag", ShaderType::Fragment)?
        .link()?;

    let outline = ShaderBuilder::new(&gl)
        .add_shader("shaders/outline.vert", ShaderType::Vertex)?
        .add_shader("shaders/outline.frag", ShaderType::Fragment)?
        .link()?;

    let window_size = window.inner_size();

    // Make sure systems using OpenGL runs on the main thread
    world.insert_non_send_resource(gl.clone());
    world.insert_resource(ShaderState::new(shader, outline));
    world.insert_resource(Camera::new(
        Camera::perspective(window_size.width, window_size.height),
        glm::vec3(0.0, 0.0, 3.0),
        glm::vec3(0.0, 0.0, -1.0),
        glm::vec3(0.0, 1.0, 0.0),
        -90.0,
        0.0,
    ));
    world.insert_resource(WindowState::new(window_size.width, window_size.height, false));
    let window = Arc::new(window);
    world.insert_resource(WinitWindow(window.clone()));
    world.insert_resource(EguiGlowRes(EguiGlow::new(&event_loop, gl, None)));
    world.insert_resource(Input::default());
    world.insert_resource(Time::default());

    let mut schedule = Schedule::default();
    schedule.add_system(ui::run_ui);
    schedule.add_system(systems::move_camera);
    schedule.add_system(systems::rotate_objects);
    schedule.add_system(systems::spawn_object);

    let mut render_schedule = Schedule::new();
    render_schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    render_schedule.add_systems((renderer::render, systems::select_object, ui::paint_ui).chain());

    let mut previous_frame_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::WindowEvent { event, .. } => {
                let camera_focused = world.resource::<WindowState>().camera_focused;
                let event_response = world.resource_mut::<EguiGlowRes>().on_event(&event);
                let consumed = if camera_focused { false } else { event_response.consumed };

                if !consumed {
                    match event {
                        WindowEvent::MouseInput { state, button: MouseButton::Right, .. } => {
                            let camera_focused =
                                &mut world.resource_mut::<WindowState>().camera_focused;
                            match state {
                                ElementState::Pressed => {
                                    window
                                        .set_cursor_grab(CursorGrabMode::Confined)
                                        .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked))
                                        .unwrap();
                                    window.set_cursor_visible(false);
                                    *camera_focused = true;
                                }
                                ElementState::Released => {
                                    window.set_cursor_grab(CursorGrabMode::None).unwrap();
                                    window.set_cursor_visible(true);
                                    *camera_focused = false;
                                }
                            }
                        }
                        WindowEvent::MouseInput { state, button, .. } => {
                            world.resource_mut::<Input>().handle_mouse_button_input(button, state);
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            world.resource_mut::<Input>().handle_mouse_move(position.into());
                        }
                        WindowEvent::KeyboardInput {
                            input: KeyboardInput { state, virtual_keycode: Some(keycode), .. },
                            ..
                        } => match keycode {
                            VirtualKeyCode::Escape => control_flow.set_exit(),
                            k => world.resource_mut::<Input>().handle_keyboard_input(k, state),
                        },
                        WindowEvent::Resized(size) => {
                            if size.width != 0 && size.height != 0 {
                                world.resource_mut::<Camera>().projection =
                                    Camera::perspective(size.width, size.height);
                                let mut ws = world.resource_mut::<WindowState>();
                                ws.width = size.width;
                                ws.height = size.height;

                                gl_surface.resize(
                                    &gl_context,
                                    size.width.try_into().unwrap(),
                                    size.height.try_into().unwrap(),
                                );
                            }
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            world.resource_mut::<Camera>().projection =
                                Camera::perspective(new_inner_size.width, new_inner_size.height);
                            let mut ws = world.resource_mut::<WindowState>();
                            ws.width = new_inner_size.width;
                            ws.height = new_inner_size.height;

                            gl_surface.resize(
                                &gl_context,
                                new_inner_size.width.try_into().unwrap(),
                                new_inner_size.height.try_into().unwrap(),
                            );
                        }
                        WindowEvent::CloseRequested => {
                            control_flow.set_exit();
                        }
                        _ => (),
                    }
                }
            }
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                if world.resource::<WindowState>().camera_focused {
                    world.resource_mut::<Input>().mouse_delta = delta;
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawEventsCleared => {
                schedule.run(&mut world);
                render_schedule.run(&mut world);

                gl_surface.swap_buffers(&gl_context).unwrap();

                let now = Instant::now();
                let delta_time = now.duration_since(previous_frame_time).as_secs_f32();
                previous_frame_time = now;
                world.resource_mut::<Time>().delta_time = delta_time;
            }
            Event::LoopDestroyed => world.resource_mut::<EguiGlowRes>().destroy(),
            _ => (),
        }
    });
}
