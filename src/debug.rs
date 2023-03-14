use std::borrow::Cow;
use std::ffi::CStr;

pub fn print_gl_info() {
    println!("Vendor: {}", get_gl_string(gl::VENDOR));
    println!("Renderer: {}", get_gl_string(gl::RENDERER));
    println!("OpenGL Version: {}", get_gl_string(gl::VERSION));
    println!("GLSL Version: {}", get_gl_string(gl::SHADING_LANGUAGE_VERSION));
}

fn get_gl_string(variant: gl::types::GLenum) -> Cow<'static, str> {
    unsafe {
        let s = gl::GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()).to_string_lossy()).unwrap_or(Cow::from(""))
    }
}
