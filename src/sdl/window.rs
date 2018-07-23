use super::api;
use libc::*;
use std::ffi::*;
use std::ptr::null_mut;

pub struct Window(*mut api::SDL_Window);

unsafe impl Sync for Window {}

impl Window {
    pub fn new<T: Into<String>>(
        title: T,
        position: Option<(i32, i32)>,
        size: (u32, u32),
        flags: u32,
    ) -> Self {
        let title = title.into();
        let title = CString::new(title).unwrap();
        let position = match position {
            Some(position) => position,
            None => (
                api::SDL_WINDOWPOS_UNDEFINED as i32,
                api::SDL_WINDOWPOS_UNDEFINED as i32,
            ),
        };
        unsafe {
            let window = api::SDL_CreateWindow(
                title.as_ptr(),
                position.0,
                position.1,
                size.0 as c_int,
                size.1 as c_int,
                flags,
            );
            if window == null_mut() {
                panic!("SDL_CreateWindow failed: {}", super::get_error());
            }
            Window(window)
        }
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
