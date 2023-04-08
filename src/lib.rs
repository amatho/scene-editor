mod commands;
mod components;
mod game_logic;
mod gl_util;
mod renderer;
mod resources;
mod shader;
mod systems;
mod ui;

use std::cell::Cell;
use std::ffi::CString;
use std::sync::mpsc::SendError;
use std::sync::{mpsc, Arc};
use std::thread;

use color_eyre::eyre::eyre;
use color_eyre::Result;
use egui_glow::EguiGlow;
use glow::{Context, HasContext as _};
use glutin::config::{Config, ConfigTemplateBuilder};
use glutin::context::{
    ContextApi, ContextAttributesBuilder, GlProfile, PossiblyCurrentContext, Version,
};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasRawWindowHandle;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub enum WinitEvent {
    WindowEvent(WindowEvent<'static>),
    ScaleFactorChanged { scale_factor: f64, new_size: PhysicalSize<u32> },
    MouseMotion((f64, f64)),
    LoopDestroyed,
}

pub fn run() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(if cfg!(debug_assertions) { Level::DEBUG } else { Level::WARN })
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|_| eyre!("setting default subscriber failed"))?;

    let (gl, gl_context, gl_config, window, event_loop) = create_glutin_window();

    let gl = Arc::new(gl);
    let window = Arc::new(window);
    // On macOS, needed to avoid program hanging after game loop thread stops
    let _wc = window.clone();

    unsafe {
        info!("Vendor: {}", gl.get_parameter_string(glow::VENDOR));
        info!("Renderer: {}", gl.get_parameter_string(glow::RENDERER));
        info!("OpenGL Version: {}", gl.get_parameter_string(glow::VERSION));
        info!("GLSL Version: {}", gl.get_parameter_string(glow::SHADING_LANGUAGE_VERSION));

        gl.enable(glow::BLEND);
        gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
    }

    let egui_glow = EguiGlow::new(&event_loop, gl.clone(), None);
    egui_glow.egui_ctx.set_pixels_per_point(window.scale_factor() as f32);
    info!("set egui pixels per point to scale factor {}", window.scale_factor(),);

    let not_current_gl_context = gl_context.make_not_current()?;
    let (event_sender, event_receiver) = mpsc::channel();

    let game_loop_thread = thread::spawn(move || {
        game_logic::run_game_loop(
            gl,
            window,
            not_current_gl_context,
            gl_config,
            egui_glow,
            event_receiver,
        )
    });
    let game_loop_thread = Cell::new(Some(game_loop_thread));

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                control_flow.set_exit();
            }
            Event::WindowEvent { event: WindowEvent::Destroyed, .. } => {
                control_flow.set_exit();
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput { virtual_keycode: Some(VirtualKeyCode::Escape), .. },
                        ..
                    },
                ..
            } => {
                control_flow.set_exit();
            }
            Event::WindowEvent {
                event: WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size },
                ..
            } => {
                let res = event_sender.send(WinitEvent::ScaleFactorChanged {
                    scale_factor,
                    new_size: *new_inner_size,
                });
                if res.is_err() {
                    get_thread_result(&game_loop_thread).unwrap();
                }
            }
            Event::WindowEvent { event, .. } => {
                let res = event_sender.send(WinitEvent::WindowEvent(event.to_static().unwrap()));
                if res.is_err() {
                    get_thread_result(&game_loop_thread).unwrap();
                }
            }
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                let res = event_sender.send(WinitEvent::MouseMotion(delta));
                if res.is_err() {
                    get_thread_result(&game_loop_thread).unwrap();
                }
            }
            Event::LoopDestroyed => {
                let _ = event_sender.send(WinitEvent::LoopDestroyed);
                if let Some(thread) = game_loop_thread.take() {
                    thread.join().unwrap().unwrap();
                }
            }
            _ => (),
        }
    });
}

fn create_glutin_window() -> (Context, PossiblyCurrentContext, Config, Window, EventLoop<()>) {
    let event_loop = winit::event_loop::EventLoop::new();
    let window_builder = WindowBuilder::new().with_title("Scene Editor");
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

    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            let s = CString::new(s).expect("failed to construct C string for gl proc address");
            gl_display.get_proc_address(&s)
        })
    };

    (gl, gl_context, gl_config, window, event_loop)
}

fn get_thread_result(cell: &Cell<Option<thread::JoinHandle<Result<()>>>>) -> Result<()> {
    if let Some(thread) = cell.take() { thread.join().unwrap() } else { Ok(()) }
}
