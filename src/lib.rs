mod renderer;
mod state;

use std::ffi::{CStr, CString};

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasRawWindowHandle;
use winit::event::{Event, WindowEvent};
use winit::window::WindowBuilder;

use crate::renderer::MainRenderer;
use crate::state::State;

pub fn run() {
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
    let raw_window_handle = window.raw_window_handle();

    let gl_display = gl_config.display();

    let context_attributes = ContextAttributesBuilder::new()
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

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);

        gl::Enable(gl::CULL_FACE);

        gl::Disable(gl::DITHER);

        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        gl::ClearColor(0.1, 0.1, 0.8, 1.0);
    }

    if let Some(renderer) = get_gl_string(gl::RENDERER) {
        println!("Running on {}", renderer.to_string_lossy());
    }

    if let Some(version) = get_gl_string(gl::VERSION) {
        println!("OpenGL Version {}", version.to_string_lossy());
    }

    if let Some(shaders_version) = get_gl_string(gl::SHADING_LANGUAGE_VERSION) {
        println!("Shaders version {}", shaders_version.to_string_lossy());
    }

    let mut state = State::new(window.inner_size().into());
    let renderer = MainRenderer;

    event_loop.run(move |event, _window_target, control_flow| {
        control_flow.set_wait();

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    if size.width != 0 && size.height != 0 {
                        state.resize(size.into());

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
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawEventsCleared => unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                state.render(&renderer);

                gl_surface.swap_buffers(&gl_context).unwrap();
            },
            _ => (),
        }
    });
}

fn get_gl_string(variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl::GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}
