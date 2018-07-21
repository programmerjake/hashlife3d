mod main;
pub mod window;
pub mod event;
pub use self::main::SDL_main;
use std::ffi::*;

#[allow(dead_code)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
mod api {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
    pub const SDL_WINDOWPOS_UNDEFINED: u32 = SDL_WINDOWPOS_UNDEFINED_MASK;
    pub const SDL_WINDOWPOS_CENTERED: u32 = SDL_WINDOWPOS_CENTERED_MASK;
}

fn get_error_message() -> String {
    unsafe { CStr::from_ptr(api::SDL_GetError()).to_str().unwrap() }.into()
}
