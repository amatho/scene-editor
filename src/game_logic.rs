use std::num::NonZeroU32;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Instant;

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ExecutorKind;
use color_eyre::Result;
use egui_glow::EguiGlow;
use glow::{Context, HasContext};
use glutin::config::Config;
use glutin::context::NotCurrentContext;
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::SwapInterval;
use glutin_winit::GlWindow;
use nalgebra_glm as glm;
use tracing::info;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyboardInput, MouseButton, WindowEvent};
use winit::window::{CursorGrabMode, Window};

use crate::components::{PointLight, Position, Scale, TransformBundle, UnloadedMesh};
use crate::resources::{
    Camera, EguiGlowRes, Input, ModelLoader, RenderSettings, Time, UiState, WinitWindow,
};
use crate::{renderer, systems, ui, WinitEvent};

pub fn run_game_loop(
    gl: Arc<Context>,
    window: Arc<Window>,
    not_current_gl_context: NotCurrentContext,
    gl_config: Config,
    egui_glow: EguiGlow,
    event_receiver: Receiver<WinitEvent>,
) -> Result<()> {
    let attrs = window.build_surface_attributes(Default::default());
    let gl_surface = unsafe { gl_config.display().create_window_surface(&gl_config, &attrs)? };
    let gl_context = not_current_gl_context.make_current(&gl_surface)?;
    gl_surface
        .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
        .unwrap();

    let mut world = World::new();

    let mut model_loader = ModelLoader::new();
    model_loader.load_models_in_dir("obj")?;
    world.spawn((
        UnloadedMesh::from(model_loader.get("plane.obj").unwrap()),
        TransformBundle {
            position: Position::new(0.0, -2.0, -15.0),
            scale: Scale::new(10.0, 1.0, 10.0),
            ..Default::default()
        },
    ));
    world.spawn((
        UnloadedMesh::from(model_loader.get("cube.obj").unwrap()),
        TransformBundle { position: Position::new(5.0, 0.0, -15.0), ..Default::default() },
    ));
    world.spawn((
        UnloadedMesh::from(model_loader.get("cube.obj").unwrap()),
        TransformBundle { position: Position::new(-5.0, 0.0, -15.0), ..Default::default() },
    ));
    world.spawn((PointLight::new(glm::vec3(1.0, 1.0, 1.0)), Position::new(0.0, 5.0, -15.0)));

    let window_size = window.inner_size();

    // Make sure systems using OpenGL runs on the main thread
    world.insert_non_send_resource(gl.clone());
    world.insert_resource(RenderSettings::new(&gl)?);
    world.insert_resource(Camera::new(
        Camera::perspective(window_size.width, window_size.height),
        glm::vec3(0.0, 0.0, 0.0),
        glm::vec3(0.0, 0.0, -1.0),
        glm::vec3(0.0, 1.0, 0.0),
        -90.0,
        0.0,
    ));
    world.insert_resource(model_loader);
    world.insert_resource(EguiGlowRes::new(egui_glow));
    world.insert_resource(WinitWindow::new(window.clone()));
    world.insert_resource(UiState::new(&window));
    world.insert_resource(Input::default());
    world.insert_resource(Time::default());

    let mut schedule = Schedule::default();
    schedule.add_system(ui::run_ui);
    schedule.add_system(systems::move_camera);
    schedule.add_system(systems::spawn_object);
    schedule.add_system(systems::select_object);
    schedule.add_system(systems::load_object_meshes);

    let mut render_schedule = Schedule::new();
    render_schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    render_schedule.add_systems((renderer::render, ui::paint_ui).chain());

    let mut previous_frame_time = Instant::now();

    'game_loop: loop {
        for event in event_receiver.try_iter() {
            match event {
                WinitEvent::WindowEvent(event) => {
                    let mut egui_glow = world.resource_mut::<EguiGlowRes>();
                    let event_response = egui_glow.on_event(&event);

                    if !event_response.consumed {
                        match event {
                            WindowEvent::MouseInput {
                                state, button: MouseButton::Right, ..
                            } => {
                                let camera_focused =
                                    &mut world.resource_mut::<UiState>().camera_focused;
                                match state {
                                    ElementState::Pressed => {
                                        window
                                            .set_cursor_grab(CursorGrabMode::Confined)
                                            .or_else(|_| {
                                                window.set_cursor_grab(CursorGrabMode::Locked)
                                            })
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
                                world
                                    .resource_mut::<Input>()
                                    .handle_mouse_button_input(button, state);
                            }
                            WindowEvent::CursorMoved { position, .. } => {
                                world.resource_mut::<Input>().handle_mouse_move(position.into());
                            }
                            WindowEvent::KeyboardInput {
                                input: KeyboardInput { state, virtual_keycode: Some(keycode), .. },
                                ..
                            } => {
                                world.resource_mut::<Input>().handle_keyboard_input(keycode, state);
                            }
                            WindowEvent::Resized(size) => {
                                resize(&gl_surface, &gl_context, &mut world, size);
                            }
                            _ => (),
                        }
                    }
                }
                WinitEvent::ScaleFactorChanged { scale_factor, new_size } => {
                    info!(
                        "scale factor changed, changing egui pixels per point to {}",
                        scale_factor
                    );
                    world
                        .resource_mut::<EguiGlowRes>()
                        .egui_ctx
                        .set_pixels_per_point(scale_factor as f32);

                    resize(&gl_surface, &gl_context, &mut world, new_size);
                }
                WinitEvent::MouseMotion(delta) => {
                    if world.resource::<UiState>().camera_focused {
                        world.resource_mut::<Input>().mouse_delta = delta;
                    }
                }
                WinitEvent::LoopDestroyed => {
                    world.resource_mut::<EguiGlowRes>().destroy();
                    break 'game_loop Ok(());
                }
            }
        }

        schedule.run(&mut world);
        render_schedule.run(&mut world);

        gl_surface.swap_buffers(&gl_context).unwrap();

        let now = Instant::now();
        let delta_time = now.duration_since(previous_frame_time).as_secs_f32();
        previous_frame_time = now;
        world.resource_mut::<Time>().delta_time = delta_time;
    }
}

fn resize(
    gl_surface: &glutin::surface::Surface<glutin::surface::WindowSurface>,
    gl_context: &glutin::context::PossiblyCurrentContext,
    world: &mut World,
    new_size: PhysicalSize<u32>,
) {
    let (width, height) = new_size.into();
    if width != 0 && height != 0 {
        // Update projection
        world.resource_mut::<Camera>().projection =
            Camera::perspective(new_size.width, new_size.height);

        // Update UI state
        let mut ws = world.resource_mut::<UiState>();
        ws.width = width;
        ws.height = height;

        // Resize surface (no-op on most platforms, needed for compatibility)
        gl_surface.resize(gl_context, width.try_into().unwrap(), height.try_into().unwrap());

        unsafe {
            // Resize viewport
            world.non_send_resource::<Arc<Context>>().viewport(0, 0, width as i32, height as i32);
        }
    }
}
