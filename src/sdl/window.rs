use super::api;
use libc::*;
use std::ffi::*;
use std::ptr::null_mut;

pub struct Window(*mut api::SDL_Window);

impl Window {
    pub fn with_position<T: Into<String>>(title: T, x: i32, y: i32, w: u32, h: u32) -> Self {
        let flags = api::SDL_WindowFlags_SDL_WINDOW_VULKAN;
        let title = title.into();
        let title = CString::new(title).unwrap();
        unsafe {
            let window = api::SDL_CreateWindow(title.as_ptr(), x, y, w as c_int, h as c_int, flags);
            if window == null_mut() {
                panic!("SDL_CreateWindow failed: {}", super::get_error_message());
            }
            Window(window)
        }
    }
    pub fn new<T: Into<String>>(title: T, w: u32, h: u32) -> Self {
        Self::with_position(
            title,
            api::SDL_WINDOWPOS_UNDEFINED as i32,
            api::SDL_WINDOWPOS_UNDEFINED as i32,
            w,
            h,
        )
    }
    pub fn get(&self) -> *mut api::SDL_Window {
        self.0
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe { api::SDL_DestroyWindow(self.0) }
    }
}
